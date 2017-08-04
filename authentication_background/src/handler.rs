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

use error::Result;

pub trait Handler<T>: Send + Sync
where
    T: Send + Sync,
{
    fn handle_present(&self, msg: &T) -> Result;
    fn handle_missing(&self) -> Result;

    fn handle(&self, msg: &Option<T>) -> Result {
        match *msg {
            Some(ref msg) => self.handle_present(msg),
            None => self.handle_missing(),
        }
    }
}
