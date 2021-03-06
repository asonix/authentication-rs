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

use std::collections::HashMap;
use std::fmt;
use error::{Error, Result};
use handler::Handler;

pub static EXIT_STR: &str = "exit";

#[derive(Clone)]
pub struct Config<'a, T>
where
    T: 'a,
{
    handlers: HashMap<&'a str, &'a Handler<T>>,
}

impl<'a, T> fmt::Debug for Config<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let keys: Vec<&str> = self.handlers.keys().map(|s| *s).collect();

        let keys = keys.join(", ");

        write!(f, "Config {}", keys)
    }
}

impl<'a, T> Default for Config<'a, T> {
    fn default() -> Self {
        Config { handlers: HashMap::new() }
    }
}

impl<'a, T> Config<'a, T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn handlers(self) -> HashMap<&'a str, &'a Handler<T>> {
        self.handlers
    }

    pub fn register_handler(&mut self, name: &'a str, handler: &'a Handler<T>) -> Result {
        if name == EXIT_STR {
            return Err(Error::ExitHandler);
        }

        if self.handlers.contains_key(name) {
            return Err(Error::DuplicateHandler(name.to_owned()));
        };

        self.handlers.insert(name, handler);

        Ok(())
    }
}
