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

extern crate zmq;
extern crate futures;
extern crate tokio_core;

mod zmq_async;
mod zmq_sync;

use futures::{Future, Stream};
use tokio_core::reactor::Core;

pub use self::zmq_sync::{ZmqReceiver, ZmqReceiverBuilder, ZmqResponder, ZmqREP};
pub use self::zmq_async::{ZmqAsyncREP, ZmqSink, ZmqStream, ZmqStreamBuilder};

pub fn run_stream() {
    let mut core = Core::new().unwrap();
    let context = zmq::Context::new();
    let sock = context.socket(zmq::REP).unwrap();
    let zmq = ZmqAsyncREP::bind(&sock, "tcp://*:5560").unwrap();

    println!("Got zmq");

    let stream = zmq.stream()
        .map(|msg| msg.to_vec())
        .map_err(|_| ())
        .and_then(|msg| String::from_utf8(msg).map_err(|_| ()))
        .map_err(|_| println!("Failed to parse string from Message"))
        .and_then(|msg| {
            println!("msg: '{}'", msg);

            zmq::Message::from_slice("hey".as_bytes()).map_err(|_| ())
        })
        .forward(zmq.sink());

    core.run(stream).unwrap();
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
