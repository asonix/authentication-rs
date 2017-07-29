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

use message::Message;
use config::EXIT_STR;

pub struct Receiver<S, T>(S)
where
    S: Iterator<Item = Message<T>>;

impl<S, T> Receiver<S, T>
where
    S: Iterator<Item = Message<T>>,
{
    pub fn new<R>(r: R) -> Self
    where
        R: IntoIterator<Item = Message<T>, IntoIter = S>,
    {
        Receiver(r.into_iter())
    }
}

impl<S, T> Iterator for Receiver<S, T>
where
    S: Iterator<Item = Message<T>>,
    T: Clone,
{
    type Item = Message<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = if let Some(msg) = self.0.next() {
            msg
        } else {
            return None;
        };

        if msg.name() == EXIT_STR {
            None
        } else {
            Some(msg)
        }
    }
}
