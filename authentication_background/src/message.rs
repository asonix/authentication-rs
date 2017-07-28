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

#[derive(Debug, Clone)]
pub struct Message<T> {
    name: &'static str,
    message: Option<T>,
    retries: i32,
}

impl<T: Clone> Message<T> {
    pub fn new(name: &'static str, message: Option<T>) -> Message<T> {
        Message::<T> {
            name: name,
            message: message,
            retries: 10,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn message(&self) -> &Option<T> {
        &self.message
    }

    pub fn retries(&self) -> i32 {
        self.retries
    }

    pub fn retry(&self) -> Self {
        Message {
            name: self.name,
            message: self.message.clone(),
            retries: self.retries - 1,
        }
    }
}
