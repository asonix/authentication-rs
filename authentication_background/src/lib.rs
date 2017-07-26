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

pub fn run<T>(config: Config<'static, T>) -> Hooks<T>
where
    T: Send + Sync + Clone,
{
    let (hook, receiver) = mpsc::channel::<Message<T>>();
    let thread_hook = hook.clone();

    let (c, p) = mpsc::channel::<CpuFuture<(), Error>>();

    let thread = thread::spawn(move || {
        let handlers = config.clone().handlers();

        let pool = CpuPool::new_num_cpus();

        for msg in receiver {
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

            let handler = handler.clone();
            let thread_hook = thread_hook.clone();

            let cpu_future: CpuFuture<(), Error> = pool.spawn_fn(move || {
                let value: FutureResult<(), Error> = handler(msg.message()).into_future();

                value.or_else(move |err| {
                    if msg.retries() > 0 {
                        println!(
                            "Task for '{}' failed with error: '{}', retrying",
                            msg.name(),
                            err
                        );
                        thread_hook.send(msg.retry())?;
                    } else {
                        println!(
                            "Task for '{}' failed permanently with error: '{}'",
                            msg.name(),
                            err
                        );
                    }
                    Ok(())
                })
            });

            c.send(cpu_future).expect("Failed to send future");
        }

        ()
    });

    let other_thread = thread::spawn(move || {
        let _: Vec<Result<(), Error>> = stream::futures_unordered(p)
            .filter(|_| false)
            .wait()
            .collect();
    });

    Hooks::new(hook, thread, other_thread)
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
