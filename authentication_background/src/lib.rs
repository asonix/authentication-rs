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

use futures::Future;
use futures::Stream;
use futures::stream;
use futures::future::{FutureResult, IntoFuture};
use futures_cpupool::{CpuPool, CpuFuture};
use std::thread;
use std::sync::mpsc;

mod message;
mod error;
mod config;
mod hooks;

pub use message::Message;
pub use error::Error;
pub use config::Config;
pub use hooks::Hooks;

pub use config::SafeHandler;

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
    fut_sender: FutSender,
) -> thread::JoinHandle<()>
where
    T: Send + Sync + Clone,
{
    thread::spawn(move || {
        let handlers = config.handlers();
        let pool = CpuPool::new_num_cpus();

        for msg in msg_receiver {
            if msg.name() == "exit" {
                break;
            }

            let handler = match handlers.get(msg.name()) {
                Some(handler) => handler,
                None => {
                    println!("No handler for message '{}'", msg.name());
                    continue;
                }
            };

            let cpu_future = future_thread(&pool, handler.clone(), msg, msg_sender.clone());

            fut_sender.send(cpu_future).expect("Failed to send future");
        }
    })
}

fn cleanup_thread(fut_receiver: FutReceiver) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for _ in stream::futures_unordered(fut_receiver)
            .filter(|_| false)
            .wait()
        {
            // do nothing
        }
    })
}

pub fn run<T>(config: Config<'static, T>) -> Hooks<T>
where
    T: Send + Sync + Clone,
{
    let (msg_sender, msg_receiver) = mpsc::channel::<Message<T>>();
    let (fut_sender, fut_receiver) = mpsc::channel::<CpuFuture<(), Error>>();

    let thread = manager_thread(config, msg_sender.clone(), msg_receiver, fut_sender);
    let other_thread = cleanup_thread(fut_receiver);

    Hooks::new(msg_sender, thread, other_thread)
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
