#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate rand;
extern crate serde;
extern crate dotenv;
extern crate frank_jwt;
extern crate rocket;
extern crate rocket_contrib;
extern crate bcrypt;
extern crate r2d2;
extern crate r2d2_diesel;

use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::{Success, Failure};
use rocket::http::Status;
use rocket::Request;
use diesel::pg::PgConnection;
use r2d2::{ Pool, PooledConnection, GetTimeout };
use r2d2_diesel::ConnectionManager;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;
pub mod routes;

type ManagedConnection = ConnectionManager<PgConnection>;
type ConnectionPool = Pool<ManagedConnection>;

lazy_static! {
    pub static ref DB_POOL: ConnectionPool = create_db_pool();
    pub static ref JWT_SECRET: String = get_jwt_secret();
}

pub struct DB(PooledConnection<ManagedConnection>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DB {
    type Error = GetTimeout;
    fn from_request(_: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Success(DB(conn)),
            Err(e) => Failure((Status::InternalServerError, e)),
        }
    }
}

fn create_db_pool() -> ConnectionPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let kept_url = database_url.clone();

    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    r2d2::Pool::new(config, manager)
        .expect(&format!(
                "Could not create connection pool for: {}",
                kept_url))
}

fn get_jwt_secret() -> String {
    dotenv().ok();

    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
