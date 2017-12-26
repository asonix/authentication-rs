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
use futures::{Async, AsyncSink, Poll, Sink, StartSend};

#[derive(Clone)]
pub struct ZmqSink<'a> {
    socket: &'a zmq::Socket,
}

impl<'a> ZmqSink<'a> {
    pub fn new(sock: &'a zmq::Socket) -> Self {
        ZmqSink { socket: sock }
    }

    fn send_message(&mut self, msg: zmq::Message) -> AsyncSink<zmq::Message> {
        let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

        match zmq::poll(&mut items, 1) {
            Ok(_) => (),
            Err(err) => {
                println!("Error in poll: {}", err);
                return AsyncSink::NotReady(msg);
            }
        };

        for item in items.iter() {
            if item.is_writable() {
                match self.socket.send(&msg, zmq::DONTWAIT) {
                    Ok(_) => {
                        return AsyncSink::Ready;
                    }
                    Err(zmq::Error::EAGAIN) => {
                        println!("Socket full, wait");
                    }
                    Err(err) => {
                        println!("Error checking item: {}", err);
                    }
                }

                break;
            }
        }

        AsyncSink::NotReady(msg)
    }

    fn flush(&mut self) -> Async<()> {
        let mut items = [self.socket.as_poll_item(zmq::POLLOUT)];

        match zmq::poll(&mut items, 1) {
            Ok(_) => Async::Ready(()),
            Err(err) => {
                println!("Error in poll: {}", err);
                Async::Ready(())
            }
        }
    }
}

impl<'a> Sink for ZmqSink<'a> {
    type SinkItem = zmq::Message;
    type SinkError = ();

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(self.send_message(item))
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.flush())
    }
}
