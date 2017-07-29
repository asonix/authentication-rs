extern crate zmq;
extern crate futures;
extern crate tokio_core;

use futures::{Stream, stream};
use tokio_core::reactor::Core;

pub struct ZmqResponder {
    responder: zmq::Socket,
}

impl ZmqResponder {
    fn connect(&self, bind_addr: &str) -> zmq::Result<()> {
        self.responder.connect(bind_addr)
    }
}

impl Iterator for ZmqResponder {
    type Item = zmq::Result<Result<String, Vec<u8>>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.responder.recv_string(0))
    }
}

pub fn run() {
    let context = zmq::Context::new();
    let responder = ZmqResponder { responder: context.socket(zmq::REP).unwrap() };
    responder.connect("tcp://localhost:5560").expect(
        "Failed connecting to responder",
    );

    let server = stream::futures_unordered(responder)
        .map_err(|_| "some err")
        .and_then(|msg| msg.map_err(|_| "some inner err"))
        .for_each(|msg| {
            println!("Got Message: {}", msg);

            Ok(())
        });

    let mut core = Core::new().unwrap();
    core.run(server).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
