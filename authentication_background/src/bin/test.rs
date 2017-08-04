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
// use std::thread;
// use std::time::Duration;

struct PrintJob;

impl Handler<i32> for PrintJob {
    fn handle_present(&self, item: &i32) -> Result {
        println!("Found number: '{}'", item);
        Ok(())
    }

    fn handle_missing(&self) -> Result {
        println!("Found nothing");
        Ok(())
    }
}

struct OtherJob;

impl Handler<i32> for OtherJob {
    fn handle_present(&self, item: &i32) -> Result {
        println!("Other number: '{}'", item);
        Ok(())
    }

    fn handle_missing(&self) -> Result {
        println!("Other nothing");
        Ok(())
    }
}

static PRINT_JOB: PrintJob = PrintJob {};
static OTHER_JOB: OtherJob = OtherJob {};

fn main() {
    let mut config = Config::new();

    config.register_handler("print", &PRINT_JOB).unwrap();
    config.register_handler("other", &OTHER_JOB).unwrap();

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
