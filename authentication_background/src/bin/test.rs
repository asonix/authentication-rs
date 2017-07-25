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

fn print_job(item: &Option<i32>) -> Result<(), Error> {
    if let Some(ref value) = *item {
        println!("Found number: '{}'", value);
    } else {
        println!("Found nothing");
    }

    Ok(())
}

fn other_job(item: &Option<i32>) -> Result<(), Error> {
    if let Some(ref value) = *item {
        println!("Other number: '{}'", value);
    } else {
        println!("Other nothing");
    }

    Ok(())
}

fn main() {
    let mut config: InitialConfig<i32> = InitialConfig::new();

    config
        .register_handler("print".to_owned(), Arc::new(print_job))
        .unwrap();

    config
        .register_handler("other".to_owned(), Arc::new(other_job))
        .unwrap();

    let config = run(config);

    let sender = config.hook();

    for num in 0..5 {
        sender
            .send(Message::new("print".to_owned(), Some(num)))
            .unwrap();
    }

    for num in 5..10 {
        sender
            .send(Message::new("other".to_owned(), Some(num)))
            .unwrap();
    }

    sender
        .send(Message::new("unused".to_owned(), None))
        .unwrap();

    cleanup(config).unwrap();
}
