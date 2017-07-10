/*
 * This file is part of Authentication.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Authentication is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Authentication is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Authentication.  If not, see <http://www.gnu.org/licenses/>.
 */

#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(dotenv_macros)]

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
extern crate regex;

use config::Config;

pub mod schema;
pub mod models;
pub mod error;
pub mod config;
pub mod webtoken;

lazy_static! {
    pub static ref CONFIG: Config = Config::initialize();
}
