# Authentication
Authentication is a user-management web service created in Rust using [Rocket](https://rocket.rs) and [Diesel](https://diesel.rs/). Please tell me how to make it more idiomatic; I'm new to this.

## Current abilities
 - `POST` to `/sign-up` with a `username` and `password` creates a `User` entry and a `VerificationCode` entry in the database and responds with a `token`.
 - `GET` to `/verify/<verification_code>` marks `User` as verified and deletes associated `VerificationCode`.
 - `POST` to `/is-authenticated` with `token` responds with whether the `token` is valid
 - `POST` to `/log-in` with a `username` and `password` responds with a `token`
 - `POST` to `/delete` with a `token` and `password` deletes a `User`

## Contributing
### Setup
Acquire [`rustup`](https://www.rustup.rs/) and use the latest nightly:

```bash
$ rustup default nightly
```

If you already have `rustup`, update to the latest nightly:

```bash
$ rustup update nightly
```

This project depends on [`PostgreSQL`](https://www.postgresql.org/), so make sure that is installed and running. Create a postgres user and a database for the application.

```bash
$ sudo -u postgres psql -c "CREATE USER your_user WITH PASSWORD 'your_users_password';"
$ sudo -u postgres psql -c "CREATE DATABASE your_database WITH OWNER your_user;"
```

Generate RSA Keys for the JSON Web Token library. The library can only understand keys in the `DER` format currently, so we'll create keys in that format.

```bash
$ mkdir -p authentication_backend/keys && cd authentication_backend/keys
$ openssl genrsa -des3 -out private.pem 2048
$ openssl rsa -in private.pem -outform DER -out private.der
$ openssl rsa -in private.der -inform DER -RSAPublicKey_out -outform DER -out public.der
```

Don't commit your keys. `authentication_backend/keys` is currently in the gitignore so you don't do this.

Copy `.env.example` to `.env` and set the required variables.

```bash
# .env
DATABASE_URL=postgres://your_user:your_users_password@localhost/your_database
JWT_PRIVATE_KEY=/path/to/authentication/authentication_backend/keys/private.der
JWT_PUBLIC_KEY=/path/to/authentication/authentication_backend/keys/public.der
```

### Running

Compile the application with:

```bash
$ cargo build
```

Run the application with 

```bash
$ cargo run
```

### Testing

Test the application with

```bash
$ cargo test
```

## License

Copyright © 2017 Riley Trautman

Authentication is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Authentication is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Authentication.

You should have received a copy of the GNU General Public License along with Authentication. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
