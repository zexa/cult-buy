extern crate dotenv;

use dotenv::dotenv;
use std::env;
use tide::prelude::*;
use tide::Request;
use rusty_money::{Money, iso};

#[async_std::main]
async fn main() -> tide::Result<()> {
    dotenv().ok();
    println!("{}", env::var("POSTGRES_USER").unwrap().as_str());

    let mut app = tide::new();
    app.at("/api/v0/users").post(post_users);
    app.at("/api/v0/listings").post(post_listings);
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct NewUser {
  email: String,
}

// TODO: Add predefined json body here
async fn post_users(mut req: Request<()>) -> tide::Result {
    let NewUser { email } = req.body_json().await?;
    Ok(format!("Welcome, {}", email).into())
}

// Only admins should be able to access this
struct NewListing<'a, T> 
where T: rusty_money::FormattableCurrency {
    name: String,
    price: Money<'a, T>,
}

// TODO: Add predefined json body here
async fn post_listings(mut req: Request<()>) -> tide::Result {
    let (name, amount): (String, i64) = req.body_json().await?;
    let price = Money::from_major(amount, iso::EUR);
    let _new_listing = NewListing {name: name.clone(), price};
    Ok(format!("Created new listing {}", name).into())
}

