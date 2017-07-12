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

use std::env;
use dotenv::dotenv;
use diesel::pg::PgConnection;
use r2d2;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use error::Result;
use config::ManagedConnection;

pub struct ConnectionPool(Pool<ManagedConnection>);

impl ConnectionPool {
    pub fn initialize() -> ConnectionPool {
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

    pub fn get(&self) -> Result<PooledConnection<ManagedConnection>> {
        Ok(self.0.get()?)
    }
}
