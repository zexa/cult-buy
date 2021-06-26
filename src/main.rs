extern crate dotenv;

use dotenv::dotenv;
use std::env;
use tide::prelude::*;
use tide::Request;
use rusty_money::{Money, iso};
use std::sync::Arc;
use tempfile::TempDir;
use std::path::Path;
use async_std::fs::OpenOptions;
use async_std::io;
use std::future::Future;
use std::pin::Pin;
use tide::Next;
use tide::StatusCode;
use tide::Response;

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv().ok();
    println!("{}", env::var("POSTGRES_USER").unwrap().as_str());

    let mut app = tide::with_state(ApplicationState::try_new()?);
    app.at("/api/v0/users").post(post_users);
    app.at("/api/v0/listings").post(post_listings);
    app.at("/api/v0/images/:file_name").with(image_validator).with(file_uploader).post(post_images);
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct NewUser {
  email: String,
}

// TODO: Add predefined json body here
async fn post_users(mut req: Request<ApplicationState>) -> tide::Result {
    let NewUser { email } = req.body_json().await?;
    Ok(format!("Welcome, {}", email).into())
}

#[derive(Debug, Deserialize)]
struct NewListing {
    name: String,
    price: u64,
}

// TODO: Add predefined json body here
async fn post_listings(mut req: Request<ApplicationState>) -> tide::Result {
    let NewListing { name, price } = req.body_json().await?;
    Ok(format!("Created new listing {}", name).into())
}

#[derive(Clone, Debug)]
struct ApplicationState {
    tempdir: Arc<TempDir>,
}


impl ApplicationState {
    fn try_new() -> Result<Self, std::io::Error> {
        Ok(Self {
            tempdir: Arc::new(tempfile::tempdir()?),
        })
    }

    fn path(&self) -> &Path {
        self.tempdir.path()
    }
}

// fn user_authenticator // middleware 

type BoxedFutureResult<'a> = Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>>;

// fn image_validator // middleware
fn image_validator<'a>(
    mut request: Request<ApplicationState>,
    next: Next<'a, ApplicationState>
//) -> BoxedFutureResult {
) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        if let Some(header) = request.header("Content-Type") {
            match header.as_str() {
                "image/png" | "image/jpeg" => {
                    println!("It do be an image"); 

                    return Ok(next.run(request).await)
                },
                _ => {}
            }
        }

        println!("Not an image");

        Ok(Response::new(StatusCode::BadRequest))
    })
}

// GET /api/v0/images/:hash
// {
//    hash,
//    created_at,
// }
// POST /api/v0/images/:file_name
// {
//   hash: 'asdasd'
// }
fn file_uploader<'a>(
    mut request: Request<ApplicationState>,
    next: Next<'a, ApplicationState>,
) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        let path = request.param("file_name")?;
        let fs_path = request.state().path().join(path);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&fs_path)
            .await?;

        let bytes_writen = io::copy(&mut request, file).await?;
        println!("File uploaded to {:?}", fs_path);
        request.set_ext(bytes_writen);

        Ok(next.run(request).await)
    })
}

async fn post_images(mut req: Request<ApplicationState>) -> tide::Result {
    println!("{}", req.ext::<u64>().unwrap());
    Ok("".into())
}

