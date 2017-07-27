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

use bcrypt::DEFAULT_COST;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use error::Result;
use dotenv::dotenv;
use self::db::DB;
use self::jwt_secret::JWTSecret;
use self::password_regex::PasswordRegex;
use self::connection_pool::ConnectionPool;

pub mod db;
mod jwt_secret;
mod password_regex;
mod connection_pool;

type ManagedConnection = ConnectionManager<PgConnection>;

pub struct Config {
    jwt_secret: JWTSecret,
    db_pool: ConnectionPool,
    password_regex: PasswordRegex,
    bcrypt_cost: u32,
}

impl Config {
    pub fn initialize() -> Self {
        Config {
            jwt_secret: JWTSecret::initialize(),
            db_pool: ConnectionPool::initialize(),
            password_regex: PasswordRegex::initialize(),
            bcrypt_cost: bcrypt_cost(),
        }
    }

    pub fn db(&self) -> Result<DB> {
        Ok(DB(self.db_pool.get()?))
    }

    pub fn jwt_secret(&self) -> &JWTSecret {
        &self.jwt_secret
    }

    pub fn password_regex(&self) -> &PasswordRegex {
        &self.password_regex
    }

    pub fn bcrypt_cost(&self) -> u32 {
        self.bcrypt_cost
    }
}

fn bcrypt_cost() -> u32 {
    dotenv().ok();

    match dotenv!("BCRYPT_COST").parse::<u32>() {
        Ok(u) => u,
        Err(_) => DEFAULT_COST,
    }
}
