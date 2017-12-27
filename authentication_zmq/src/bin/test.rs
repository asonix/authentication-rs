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
extern crate tokio_core;
extern crate futures;
extern crate authentication_zmq;

use std::rc::Rc;

use tokio_core::reactor::Core;
use futures::stream::iter_ok;
use futures::{Future, Stream};

use authentication_zmq::ZmqREP;
use authentication_zmq::RepBuilder;

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
        let zmq_async = RepBuilder::new(Rc::new(sock))
            .connect("tcp://localhost:5560")
            .unwrap();

        let mut core = Core::new().unwrap();

        let stream = iter_ok(5..10)
            .and_then(|req| {
                println!("sending: {}", req);
                zmq_async
                    .send(zmq::Message::from_slice("Hello".as_bytes()).unwrap())
                    .map(move |message| (req, message))
            })
            .for_each(|(req, message)| {
                println!("Received reply {} {}", req, message.as_str().unwrap());
                Ok(())
            });

        core.run(stream).unwrap();
    }
}
