# Authentication Background
Authentication Background is the background jobs library used by the Authentication web application, though it is portable and can be used in any situation. This library uses [Futures](https://tokio.rs/docs/getting-started/futures/) to manage its thread pool.

## Usage
```Rust
// src/main.rs

extern crate authentication_background;

use authentication_background::{Config, Message, Result, Handler, run};

type MyMessage = i32;

struct SomeHandler;

impl Handler<MyMessage> for SomeHandler {
    fn handle_present(&self, msg: &MyMessage) -> Result {
        println!("Got: {}", item);
        Ok(())
    }

    fn handle_missing(&self) -> Result {
        Ok(())
    }
}

static SOME_HANDLER: SomeHandler = SomeHandler {};

fn main() {
    // Create a new configuration for a background job that sends MyMessages
    let config = Config::new::<MyMessage>();

    // Register SOME_HANDLER as a job handler under the name "some_handler"
    config.register_handler("some_handler", &SOME_HANDLER).unwrap();

    // Actually fire up the background job threads
    let hooks = run(config);

    // Get a sender to send jobs to the background job threads
    let sender = hooks.hook();

    // Send a message of Some(5) to the background job threads.
    // This message will be processed by the handler registered under
    // the "some_handler" name.
    sender.send(Message::new("some_handler", Some(5))).unwrap();

    // Tear down the background job threads
    hooks.cleanup().unwrap();
}
```

## License

Copyright © 2017 Riley Trautman

Authentication is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Authentication is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Authentication.

You should have received a copy of the GNU General Public License along with Authentication. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
