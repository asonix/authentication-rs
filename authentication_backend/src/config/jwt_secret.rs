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
use dotenv::dotenv;
use jwt;
use jwt::{Header, Validation};
use error::Result;
use serde::{Serialize, Deserialize};

pub struct JWTSecret {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

impl JWTSecret {
    pub fn initialize() -> JWTSecret {
        dotenv().ok();

        JWTSecret {
            private_key: JWTSecret::read_file(dotenv!("JWT_PRIVATE_KEY")),
            public_key: JWTSecret::read_file(dotenv!("JWT_PUBLIC_KEY")),
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

    pub fn encode<T>(&self, header: &Header, claims: &T) -> Result<String>
    where
        T: Serialize,
    {
        let token = jwt::encode(header, claims, &self.private_key)?;

        Ok(token)
    }

    pub fn decode<T>(&self, token: &str, validation: &Validation) -> Result<T>
    where
        for<'a> T: Deserialize<'a>,
    {
        let token_data = jwt::decode::<T>(token, &self.public_key, validation)?;

        Ok(token_data.claims)
    }
}
