# Authentication
Authentication is a user-management web service created in Rust using [Rocket](https://rocket.rs) and [Diesel](https://diesel.rs/). Please tell me how to make it more idiomatic; I'm new to this.

## Current abilities
### Server binary
#### Usage
```bash
$ cargo run --bin server
```
#### Information
This application accepts data as JSON. It can create, verify, authenticate, and delete users.
 - `POST /sign-up`
    - INPUT: **username** and **password**
    - Creates a **User** entry and a **VerificationCode** entry in the database.
    - OUTPUT: **user_id** and **username**
 - `POST /log-in`
    - INPUT: **username** and **password**
    - OUTPUT: **user_token** and **renewal_token**
 - `POST /is-authenticated`
    - INPUT: **auth**
    - OUTPUT: Whether or not **auth** is valid
 - `POST /users/<target_user>/delete`
    - INPUT: **auth**
    - Deletes **target_user** if **user_token** represents **target_user** or if **user_token** represents and admin.
 - `POST /users/<target_user>/grant/<permission>`
    - INPUT: **auth**, **target_user**, and **permission**
    - If **auth** represents an admin, gives **target_user** the **permission**.
 - `POST /users/<target_user>/revoke/<permission>`
    - INPUT: **auth**, **target_user**, and **permission**
    - If **auth** represents an admin, revokes the **permission** from **target_user**.
 - `POST /renew-token`
    - INPUT: **renewal_token**
    - OUTPUT: **user_token** and **renewal_token**
 - `GET /verify/<verification_code>`
    - INPUT: **verification_code**
    - Marks **User** as verified and deletes associated **VerificationCode**.
 - `POST /permissions`
    - INPUT: **auth** and **permission_name**
    - If **auth** represents and admin, creates a new **Permission** with **permission_name**
 - `POST /permissions/<permission>/delete`
    - INPUT: **auth** and **permission**
    - If **auth** represents and admin, deletes the **permission**

### MakeAdmin binary
#### Usage
```bash
$ cargo run --bin make_admin <username>
```
#### Information
This binary accepts a **username** as a commandline argument and makes that user an admin.

### VerifyUser binary
#### Usage
```bash
$ cargo run --bin verify_user <username>
```
#### Information
This binary accepts a **username** as a commandline argument and verifies that user.

### CreateUser binary
#### Usage
```bash
$ cargo run --bin create_user <username> <password>
```
#### Information
This binary accepts a **username** and **password** as commandline arguments and creates a user with that information.

#### Information
This binary accepts a **username** as a commandline argument and verifies that user.

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
BCRYPT_COST=4
```

The `BCRYPT_COST` in the environment is optional. If unspecified, BCrypt will use the `DEFAULT_COST` which is 12 at the time of writing. This value exists on a scale of 4 to 31. To make testing quicker, smaller values can be used. For production systems, larger values should be used.

Install [`diesel_cli`](http://diesel.rs/guides/getting-started/) and make sure your global rust binaries are in your path.

Installing:
```bash
$ cargo install diesel_cli
```

Setting path in `~/.bashrc` for bash:
```bash
# ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```

Setting path in `~/.zshenv` for zsh:
```zsh
# ~/.zshenv
path=(~/.cargo/bin $path[@])
```

You may need to restart your shell for changes to take effect.

```bash
$ exec $SHELL
```

Run the existing migrations to bring your database up to speed.

```bash
$ cd authentication_backend
$ diesel migration run
```

### Running

Compile the application with:

```bash
$ cargo build
```

Run the application with 

```bash
$ cargo run --bin server
```

### Testing

Test the application with. Currently there are tests for the authentication_backend and authentication_rocket packages. More tests will come.

```bash
$ cargo test
```

## License

Copyright © 2017 Riley Trautman

Authentication is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Authentication is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Authentication.

You should have received a copy of the GNU General Public License along with Authentication. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
