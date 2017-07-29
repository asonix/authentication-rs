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

use zmq;
use futures::{Async, Poll, Stream};

pub struct ZmqSingle {
    socket: zmq::Socket,
}

impl ZmqSingle {
    pub fn new(sock: zmq::Socket) -> Self {
        ZmqSingle { socket: sock }
    }

    fn next_message(&self) -> Async<Option<zmq::Message>> {
        let mut items = [self.socket.as_poll_item(zmq::POLLIN)];

        // Don't block waiting for an item to become ready
        match zmq::poll(&mut items, 0) {
            Ok(_) => (),
            Err(err) => {
                println!("Error polling: {}", err);
                return Async::NotReady;
            }
        };

        let mut msg = zmq::Message::new().unwrap();

        for item in items.iter() {
            if item.is_readable() {
                match self.socket.recv(&mut msg, 0) {
                    Ok(_) => return Async::Ready(Some(msg)),
                    Err(err) => {
                        println!("Error checking item: {}", err);
                    }
                }
            }
        }

        println!("not ready");

        Async::NotReady
    }
}

impl Stream for ZmqSingle {
    type Item = zmq::Message;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<zmq::Message>, ()> {
        Ok(self.next_message())
    }
}

pub struct ZmqMany<'a> {
    sockets: Vec<&'a zmq::Socket>,
}

impl<'a> ZmqMany<'a> {
    pub fn new() -> Self {
        ZmqMany { sockets: Vec::new() }
    }

    pub fn add_socket(&mut self, sock: &'a zmq::Socket) {
        self.sockets.push(sock);
    }

    fn next_message(&self) -> Async<Option<zmq::Message>> {
        let mut items = self.sockets
            .clone()
            .iter()
            .map(|sock| sock.as_poll_item(zmq::POLLIN))
            .collect::<Vec<_>>();

        // Don't block waiting for an item to become ready
        match zmq::poll(&mut items, 0) {
            Ok(_) => (),
            Err(_) => {
                // debug!("Error polling: {}", err);
                return Async::NotReady;
            }
        };

        let mut msg = zmq::Message::new().unwrap();

        for (index, item) in items.iter().enumerate() {
            if item.is_readable() {
                match self.sockets[index].recv(&mut msg, 0) {
                    Ok(_) => return Async::Ready(Some(msg)),
                    Err(_) => {
                        // debug!("Error checking item: {}", err);
                    }
                };
            };
        }

        Async::NotReady
    }
}

impl<'a> Stream for ZmqMany<'a> {
    type Item = zmq::Message;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<zmq::Message>, ()> {
        Ok(self.next_message())
    }
}
