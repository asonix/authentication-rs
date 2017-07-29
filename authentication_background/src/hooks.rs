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

use std::thread;
use std::sync::mpsc;
use message::Message;
use config::EXIT_STR;
use error::Error;

#[derive(Debug)]
pub struct Hooks<T> {
    hook: mpsc::Sender<Message<T>>,
    handle: thread::JoinHandle<()>,
    other_handle: thread::JoinHandle<()>,
}

impl<T: Send + Sync + Clone> Hooks<T> {
    pub fn new(
        sender: mpsc::Sender<Message<T>>,
        handle: thread::JoinHandle<()>,
        other_handle: thread::JoinHandle<()>,
    ) -> Self {
        Hooks {
            hook: sender,
            handle: handle,
            other_handle: other_handle,
        }
    }

    pub fn cleanup(self) -> Result<(), Error> {
        let Hooks {
            other_handle,
            handle,
            hook,
        } = self;

        hook.send(Message::new(EXIT_STR, None))?;

        handle.join()?;
        other_handle.join()?;

        Ok(())
    }

    pub fn hook(&self) -> mpsc::Sender<Message<T>> {
        self.hook.clone()
    }
}
