#![feature(plugin, custom_derive, custom_attribute)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate dotenv;
extern crate jsonwebtoken as jwt;
extern crate bcrypt;
extern crate r2d2;
extern crate r2d2_diesel;

use config::Config;

pub mod schema;
pub mod models;
pub mod error;
pub mod config;
pub mod webtoken;

lazy_static! {
    pub static ref CONFIG: Config = Config::initialize();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
