extern crate dotenv;

use dotenv::dotenv;
use tide::prelude::*;
use tide::Request;
use std::sync::Arc;
use async_std::fs::OpenOptions;
use async_std::io;
use std::future::Future;
use std::pin::Pin;
use tide::Next;
use tide::StatusCode;
use tide::Response;
use tide::log;
use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;
use std::path::PathBuf;
use fs_extra::file::move_file;
use fs_extra::file::CopyOptions;

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv().ok();

    let mut app = tide::with_state(ApplicationState::try_new()?);
    log::start();
    app.at("/api/v0/users").post(post_users);
    app.at("/api/v0/listings").post(post_listings);
    app.at("/api/v0/images/:file_name")
        .with(image_validator)
        .with(file_uploader)
        .post(post_images);
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct NewUser {
  email: String,
}

async fn post_users(mut req: Request<ApplicationState>) -> tide::Result {
    let NewUser { email } = req.body_json().await?;
    Ok(format!("Welcome, {}", email).into())
}

#[derive(Debug, Deserialize)]
struct NewListing {
    name: String,
    price: u64,
}

async fn post_listings(mut req: Request<ApplicationState>) -> tide::Result {
    let NewListing { name, price } = req.body_json().await?;
    Ok(format!("Created new listing {} {}", name, price).into())
}

#[derive(Clone, Debug)]
struct ApplicationState {
    tempdir: Arc<PathBuf>,
    permdir: Arc<PathBuf>,
}


impl ApplicationState {
    fn try_new() -> Result<Self, std::io::Error> {
        Ok(Self {
            tempdir: Arc::new(PathBuf::from("/cult-buy/temp/")),
            // TODO: FIX THIS REEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE
            permdir: Arc::new(PathBuf::from("/cult-buy/files/")),
        })
    }

    // TODO: Rename to tempdir()
    fn tempdir(&self) -> &PathBuf {
        &self.tempdir
    }

    fn permdir(&self) -> &PathBuf {
        &self.permdir
    }
}

type BoxedFutureResult<'a> = Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>>;

fn image_validator<'a>(
    mut request: Request<ApplicationState>,
    next: Next<'a, ApplicationState>
) -> BoxedFutureResult {
    Box::pin(async {
        let body = request.body_bytes().await?;
        let file_name = request.param("file_name")?;
        let body_len = body.len();

        if body_len > 10_000_000 {
            log::info!("Image {} too big. Size: {}.", file_name, body_len);

            return Ok(Response::new(StatusCode::BadRequest))
        };

        // TODO: Reading an image should not be blocking.
        if let Err(_) = image::load_from_memory(&body[..]) {
            log::info!("Image {} is not an image", file_name);

            return Ok(Response::new(StatusCode::BadRequest))
        }


        if let Some(header) = request.header("Content-Type") {
            match header.as_str() {
                "image/png" | "image/jpeg" => {
                    request.set_body(body);

                    return Ok(next.run(request).await)
                },
                _ => {}
            }
        }

        log::info!("Wrong Content-type");

        Ok(Response::new(StatusCode::BadRequest))
    })
}

#[derive(Debug, Clone)]
struct File {
    hash: Uuid,
    name: String,
    path: PathBuf,
    size: u64,
    content_type: String,
    created_at: DateTime<Utc>,
    // TODO: Add url here maybe?
}

impl File {
    pub fn new(name: String, path: PathBuf, size: u64, content_type: String) -> Self {
        Self {
            hash: Uuid::new_v4(),
            name,
            path,
            size,
            content_type,
            created_at: Utc::now(),
        }
    }

    pub fn hash(&self) -> &Uuid {
        &self.hash
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size 
    }

    // /home/zexa/Projects/cult-buy/files/a1c69e0d-a6d7-4594-acf3-188ac29e5c16
    pub fn take_ownership(&mut self, folder: &PathBuf) -> Result<(), std::io::Error> {
        let path = folder.join(self.hash.to_string());

        move_file(self.path(), &path, &CopyOptions::new());
        self.path = path;

        Ok(())
    }
}

fn file_uploader<'a>(
    mut request: Request<ApplicationState>,
    next: Next<'a, ApplicationState>,
) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        let path = request.param("file_name")?;
        let fs_path = request.state().tempdir().join(path.clone());

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&fs_path)
            .await?;

        let content_type = request.header("Content-Type").unwrap().to_string();

        let file = File::new(
            path.to_string(), 
            fs_path, 
            io::copy(&mut request, file).await?,
            content_type
        );

        log::info!("File uploaded: {:?}", file);

        request.set_ext(file);

        Ok(next.run(request).await)
    })
}

#[derive(Debug, Clone)]
struct Image {
    hash: Uuid,
    file: File,
    created_at: DateTime<Utc>,
}

impl Image {
    pub fn new(mut file: File, folder: &PathBuf) -> Self {
        file.take_ownership(folder).unwrap();

        Self {
            hash: Uuid::new_v4(),
            file,
            created_at: Utc::now(),
        }
    } 

    pub fn hash(&self) -> &Uuid {
        &self.hash
    }

    pub fn file(&self) -> &File {
        &self.file
    }
}

async fn post_images(req: Request<ApplicationState>) -> tide::Result {
    let image = Image::new(
        req.ext::<File>().unwrap().clone(),
        req.state().permdir()
    );

    log::info!("Image uploaded: {:?}", image);

    Ok("".into())
}



struct User {
    hash: Uuid,
    email: String,
    // session: Vec<Session>
    // login_codes: Vec<LoginCode>
    // shipping_address: Option<ShippingAddress>
    // cart: Option<Cart>,
    // created_at: DateTime,
    // modified_at: DateTime,
    // has_signed_up_for_newsletter: bool,
}

