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

#![feature(conservative_impl_trait)]

extern crate zmq;
extern crate futures;
extern crate tokio_core;

mod async;
mod zmq_sync;

use std::rc::Rc;

use futures::Future;
use tokio_core::reactor::Core;

pub use self::zmq_sync::{ZmqReceiver, ZmqReceiverBuilder, ZmqResponder, ZmqREP};
pub use self::async::{RepHandler, RepBuilder, RepServer, RepClient, ZmqSink, ZmqStream,
                      ZmqResponse};

#[derive(Clone)]
pub struct Echo;

#[derive(Debug)]
pub enum Error {
    One,
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::One
    }
}

impl RepHandler for Echo {
    type Request = zmq::Message;
    type Response = zmq::Message;
    type Error = Error;

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(futures::future::ok(req))
    }
}

pub fn run_stream() {
    let mut core = Core::new().unwrap();
    let context = zmq::Context::new();
    let sock = context.socket(zmq::REP).unwrap();
    let zmq = RepBuilder::new(Rc::new(sock))
        .handler(Echo {})
        .bind("tcp://*:5560")
        .unwrap();

    println!("Got zmq");

    core.run(zmq.runner()).unwrap();
}

pub fn run() {
    let context = zmq::Context::new();
    let sock = context.socket(zmq::REP).unwrap();
    let zmq = ZmqREP::bind(&sock, "tcp://*:5560").unwrap();

    println!("Got zmq");

    for msg in zmq.incomming() {
        println!("Got something");

        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                println!("Error: '{}'", err);
                continue;
            }
        };

        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                let err = match String::from_utf8(err) {
                    Ok(err) => err,
                    _ => {
                        println!("Uknown ZMQ Error");
                        continue;
                    }
                };

                println!("Error: '{}'", err);
                continue;
            }
        };

        println!("Got Message: {}", msg);

        let _ = zmq.send("hey");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
