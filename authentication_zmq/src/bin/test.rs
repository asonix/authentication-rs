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
extern crate authentication_zmq;
extern crate futures;
extern crate tokio_core;

use zmq::Message;
use authentication_zmq::{ZmqAsyncREP, ZmqREP};
use futures::stream::iter_ok;
use futures::{Future, Sink, Stream};
use tokio_core::reactor::Core;

fn main() {
    let context = zmq::Context::new();
    {
        let sock = context.socket(zmq::REQ).unwrap();
        let zmq_sync = ZmqREP::connect(&sock, "tcp://localhost:5560").unwrap();

        for req in 0..5 {
            println!("sending: {}", req);
            zmq_sync.send("Hello").unwrap();

            let message = zmq_sync.incomming().next().unwrap().unwrap().unwrap();
            println!("Received reply {} {}", req, message);
        }
    }

    {
        let sock = context.socket(zmq::REQ).unwrap();
        let zmq_async = ZmqAsyncREP::connect(&sock, "tcp://localhost:5560").unwrap();
        let mut core = Core::new().unwrap();

        for req in 0..5 {
            let receive = zmq_async
                .sink()
                .send(Message::from_slice("hello".as_bytes()).unwrap())
                .and_then(|_| zmq_async.stream().into_future().map_err(|_| ()))
                .and_then(|(msg, _)| match msg {
                    Some(msg) => {
                        String::from_utf8(msg.to_vec())
                            .map_err(|_| println!("Failed to parse string from Message"))
                            .map(|msg| println!("Received reply {} {}", req, msg))
                    }
                    None => Err(()),
                });

            core.run(receive).unwrap();
        }
    }
}
