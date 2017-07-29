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

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

use futures::Future;
use futures::Stream;
use futures::stream;
use futures::future::{FutureResult, IntoFuture};
use futures_cpupool::{CpuPool, CpuFuture};
use tokio_core::reactor::Core;
use std::thread;
use std::sync::mpsc;

mod message;
mod error;
mod config;
mod hooks;

pub use message::Message;
pub use error::{Error, Result};
pub use config::{Config, SafeHandler};
pub use hooks::Hooks;

use config::EXIT_STR;

pub type MsgSender<T> = mpsc::Sender<Message<T>>;
pub type MsgReceiver<T> = mpsc::Receiver<Message<T>>;

type FutSender = mpsc::Sender<CpuFuture<(), Error>>;
type FutReceiver = mpsc::Receiver<CpuFuture<(), Error>>;

fn future_thread<T>(
    pool: &CpuPool,
    handler: SafeHandler<'static, T>,
    msg: Message<T>,
    msg_sender: MsgSender<T>,
) -> CpuFuture<(), Error>
where
    T: 'static + Send + Sync + Clone,
{
    pool.spawn_fn(move || {
        let value: FutureResult<(), Error> = handler(msg.message()).into_future();

        value.or_else(move |err| {
            if msg.retries() > 0 {
                println!(
                    "Task for '{}' failed with error: '{}', retrying",
                    msg.name(),
                    err
                );
                msg_sender.send(msg.retry())?;
            } else {
                println!(
                    "Task for '{}' failed permanently with error: '{}'",
                    msg.name(),
                    err
                );
            }
            Ok(())
        })
    })
}

fn manager_thread<T>(
    config: Config<'static, T>,
    msg_sender: MsgSender<T>,
    msg_receiver: MsgReceiver<T>,
) -> thread::JoinHandle<()>
where
    T: Send + Sync + Clone,
{
    thread::spawn(move || {
        let handlers = config.handlers();
        let pool = CpuPool::new_num_cpus();

        let messages = msg_receiver.iter().map(|msg| Ok(msg));
        let server = stream::futures_unordered(messages)
            .and_then(|msg| {
                let handler = match handlers.get(msg.name()) {
                    Some(handler) => handler,
                    None => return Err(format!("No handler for message '{}'", msg.name())),
                };

                handler(msg.message()).map_err(|_| msg)
            })
            .or_else(|msg| if msg.retries() > 0 {
                println!(
                    "Task for '{}' failed",
                    msg.name(),
                );
                msg_sender.send(msg.retry())?;
            } else {
                println!(
                    "Task for '{}' failed permanently",
                    msg.name(),
                );
            })
            .for_each(|_| ());

        let mut core = Core::new().unwrap();

        core.run(server).unwrap();
    })
}

pub fn run<T>(config: Config<'static, T>) -> Hooks<T>
where
    T: Send + Sync + Clone,
{
    let (msg_sender, msg_receiver) = mpsc::channel::<Message<T>>();

    let thread = manager_thread(config, msg_sender.clone(), msg_receiver);

    Hooks::new(msg_sender, thread)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_works() {
        let config: Config<i32> = Config::new();

        let hooks = run(config);

        let result = hooks.cleanup();

        assert!(result.is_ok(), "Failed to perform job lifecycle");
    }
}
