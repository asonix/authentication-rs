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

fn main() {
    let context = zmq::Context::new();
    let requester = context.socket(zmq::REQ).unwrap();

    requester.connect("tcp://localhost:5560").expect(
        "Failed to connect requester",
    );

    for req in 0..10 {
        println!("sending: {}", req);
        requester.send("Hello".as_bytes(), 0).unwrap();

        let message = requester.recv_msg(0).unwrap();
        println!("Received reply {} {}", req, message.as_str().unwrap());
    }
}
