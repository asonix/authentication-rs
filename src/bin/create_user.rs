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

extern crate authentication_backend;

use std::env;
use authentication_backend::Authenticatable;
use authentication_backend::User;
use authentication_backend::UserTrait;

fn main() {
    let mut args = env::args();

    if args.len() != 3 {
        panic!("Wrong number of arguments");
    }

    let _executable: Option<String> = args.next();

    let uname: String = args.next().expect("Failed to get username from arguments");
    let password: String = args.next().expect("Failed to get password from arguments");

    let auth = Authenticatable::UserAndPass {
        username: &uname,
        password: &password, // "ThisIsAP4ssw0rt$.",
    };

    let user = User::create(&auth);

    let user = match user {
        Ok(user) => user,
        Err(error) => panic!("We didn't get a user: '{}'", error),
    };

    println!("Created a user with username: '{}'", user.username());
}
