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
extern crate futures_cpupool;

mod zmq_stream;

use futures::{Future, Stream};
use futures_cpupool::CpuPool;
pub use self::zmq_stream::{ZmqSingle, ZmqMany};

#[derive(Clone)]
pub struct ZmqReceiver<'a> {
    receiver: &'a zmq::Socket,
}

impl<'a> ZmqReceiver<'a> {
    pub fn new(receiver: &'a zmq::Socket) -> ZmqReceiverBuilder<'a> {
        ZmqReceiverBuilder { receiver: receiver }
    }
}

pub struct ZmqReceiverBuilder<'a> {
    receiver: &'a zmq::Socket,
}

impl<'a> ZmqReceiverBuilder<'a> {
    pub fn bind(self, bind_addr: &str) -> zmq::Result<ZmqReceiver<'a>> {
        self.receiver.bind(bind_addr)?;

        Ok(ZmqReceiver { receiver: self.receiver })
    }

    pub fn connect(self, bind_addr: &str) -> zmq::Result<ZmqReceiver<'a>> {
        self.receiver.connect(bind_addr)?;

        Ok(ZmqReceiver { receiver: self.receiver })
    }
}

impl<'a> Iterator for ZmqReceiver<'a> {
    type Item = zmq::Result<Result<String, Vec<u8>>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.receiver.recv_string(0))
    }
}

pub struct ZmqResponder<'a> {
    responder: &'a zmq::Socket,
}

impl<'a> ZmqResponder<'a> {
    pub fn new(responder: &'a zmq::Socket) -> Self {
        ZmqResponder { responder: responder }
    }

    pub fn send(&self, msg: &str) -> zmq::Result<()> {
        self.responder.send(msg.as_bytes(), 0)
    }
}

pub struct ZmqREP<'a> {
    receiver: ZmqReceiver<'a>,
    responder: ZmqResponder<'a>,
}

impl<'a> ZmqREP<'a> {
    pub fn bind(sock: &'a zmq::Socket, addr: &str) -> zmq::Result<Self> {
        let responder = ZmqResponder::new(&sock);
        let receiver = ZmqReceiver::new(&sock).bind(addr)?;

        Ok(ZmqREP {
            receiver: receiver,
            responder: responder,
        })
    }

    pub fn connect(sock: &'a zmq::Socket, addr: &str) -> zmq::Result<Self> {
        let responder = ZmqResponder::new(&sock);
        let receiver = ZmqReceiver::new(&sock).connect(addr)?;

        Ok(ZmqREP {
            receiver: receiver,
            responder: responder,
        })
    }

    pub fn incomming(&self) -> ZmqReceiver {
        self.receiver.clone()
    }

    pub fn send(&self, msg: &str) -> zmq::Result<()> {
        self.responder.send(msg)
    }
}

pub fn run_stream() {
    let context = zmq::Context::new();
    let sock = context.socket(zmq::REP).unwrap();
    sock.bind("tcp://*:5560").unwrap();

    println!("Before stream");

    let stream = ZmqSingle::new(sock)
        .map(|msg| msg.to_vec())
        .map_err(|_| Vec::new())
        .and_then(|msg| String::from_utf8(msg).map_err(|e| e.into_bytes()))
        .or_else(|err| {
            println!("Failed to parse string from Message");

            Err(err)
        })
        .for_each(|msg| {
            println!("msg: '{}'", msg);

            Ok(())
        });

    let pool = CpuPool::new_num_cpus();
    let res = pool.spawn_fn(move || stream);

    res.wait();

    println!("After stream");
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
