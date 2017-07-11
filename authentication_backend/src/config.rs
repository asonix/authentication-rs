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

use std::fs::File;
use std::io::Read;
use std::env;
use dotenv::dotenv;
use webtoken::Claims;
use jwt;
use jwt::{Header, Validation};
use diesel::pg::PgConnection;
use r2d2;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use error::Result;
use regex::Regex;

type ManagedConnection = ConnectionManager<PgConnection>;

pub struct ConnectionPool(Pool<ManagedConnection>);

impl ConnectionPool {
    fn initialize() -> ConnectionPool {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let kept_url = database_url.clone();

        let config = r2d2::Config::default();
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        ConnectionPool(Pool::new(config, manager).expect(&format!(
            "Could not create connection pool for: {}",
            kept_url
        )))
    }

    fn get(&self) -> Result<PooledConnection<ManagedConnection>> {
        Ok(self.0.get()?)
    }
}

pub struct DB(PooledConnection<ManagedConnection>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
    }
}

pub struct PasswordRegex {
    numbers: Regex,
    symbols: Regex,
    upper: Regex,
    lower: Regex,
}

impl PasswordRegex {
    fn initialize() -> Self {
        PasswordRegex {
            numbers: Regex::new("[0-9]").unwrap(),
            symbols: Regex::new("[!@#$%^&*();\\\\/|<>\"'_+\\-\\.,?=]").unwrap(),
            upper: Regex::new("[A-Z]").unwrap(),
            lower: Regex::new("[a-z]").unwrap(),
        }
    }

    pub fn numbers(&self) -> &Regex {
        &self.numbers
    }

    pub fn symbols(&self) -> &Regex {
        &self.symbols
    }

    pub fn upper(&self) -> &Regex {
        &self.upper
    }

    pub fn lower(&self) -> &Regex {
        &self.lower
    }
}

pub struct JWTSecret {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

impl JWTSecret {
    fn initialize() -> JWTSecret {
        dotenv().ok();

        JWTSecret {
            private_key: JWTSecret::read_file(env!("JWT_PRIVATE_KEY")),
            public_key: JWTSecret::read_file(env!("JWT_PUBLIC_KEY")),
        }
    }

    fn read_file(filename: &str) -> Vec<u8> {
        let mut f = File::open(filename).expect(&format!("File '{}' does not exist", filename));
        let mut contents: Vec<u8> = Vec::new();

        f.read_to_end(&mut contents).expect(&format!(
            "Failed to read file '{}'",
            filename
        ));

        contents
    }

    pub fn encode(&self, header: &Header, claims: &Claims) -> Result<String> {
        let token = jwt::encode(header, claims, &self.private_key)?;
        println!("Created token: '{}'", &token);

        Ok(token)
    }

    pub fn decode(&self, token: &str, validation: &Validation) -> Result<Claims> {
        println!("Processing token: '{}'", &token);

        let token_data = jwt::decode::<Claims>(token, &self.public_key, validation)?;

        Ok(token_data.claims)
    }
}

pub struct Config {
    jwt_secret: JWTSecret,
    db_pool: ConnectionPool,
    password_regex: PasswordRegex,
}

impl Config {
    pub fn initialize() -> Self {
        Config {
            jwt_secret: JWTSecret::initialize(),
            db_pool: ConnectionPool::initialize(),
            password_regex: PasswordRegex::initialize(),
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
}
