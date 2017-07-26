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

#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate rocket;
extern crate rocket_contrib;
extern crate authentication_backend;
extern crate authentication_background;

mod routes;
mod jobs;
mod controllers;
mod error;
mod auth_response;
mod input_types;

use std::sync::Mutex;

pub fn launch() -> () {
    let mut config: authentication_background::Config<i32> =
        authentication_background::Config::new();

    jobs::register_jobs(&mut config);

    let config = config;

    let hooks = authentication_background::run(config);

    let error = rocket::ignite()
        .mount(
            "/",
            routes![
                routes::users::sign_up,
                routes::users::log_in,
                routes::users::is_authenticated,
                routes::users::delete,
                routes::users::grant_permission,
                routes::users::revoke_permission,
                routes::webtokens::renew,
                routes::verification_codes::verify,
                routes::permissions::create,
                routes::permissions::delete,
            ],
        )
        .manage(Mutex::new(hooks.hook()))
        .launch();

    hooks.cleanup().unwrap();

    panic!("Launch failed! Error: {}", error)
}
