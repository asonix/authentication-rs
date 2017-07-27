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

extern crate authentication_background;

use authentication_background::*;
use std::sync::Arc;
// use std::thread;
// use std::time::Duration;

fn print_job(item: &Option<i32>) -> Result {
    if let Some(ref value) = *item {
        println!("Found number: '{}'", value);
    } else {
        println!("Found nothing");
    }

    Ok(())
}

fn other_job(item: &Option<i32>) -> Result {
    if let Some(ref value) = *item {
        println!("Other number: '{}'", value);
    } else {
        println!("Other nothing");
    }

    Ok(())
}

fn main() {
    let mut config: Config<i32> = Config::new();

    config
        .register_handler("print", Arc::new(print_job))
        .unwrap();

    config
        .register_handler("other", Arc::new(other_job))
        .unwrap();

    let hooks = run(config);

    let sender = hooks.hook();

    let mid = 5;

    for num in 0..mid {
        sender.send(Message::new("print", Some(num))).unwrap();
    }

    for num in mid..(mid * 2) {
        sender.send(Message::new("other", Some(num))).unwrap();
    }

    sender.send(Message::new("unused", None)).unwrap();

    // thread::sleep(Duration::from_secs(1));

    hooks.cleanup().unwrap();
}