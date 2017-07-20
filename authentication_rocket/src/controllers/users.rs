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

use authentication_backend::Error as BackendError;
use authentication_backend::{ToAuth, Admin, User, UserTrait};
use routes::Response;
use auth_response::AuthResponse;

pub fn sign_up<T>(auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::create(auth)?;

    Ok(AuthResponse::new("User created", user))
}

pub fn log_in<T>(auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate_session(auth)?;

    let token = user.create_webtoken().ok();

    Ok(AuthResponse::new("Authenticated", token))
}

pub fn is_authenticated<T>(auth: &T) -> Response
where
    T: ToAuth,
{
    User::authenticate(auth)?;

    Ok(AuthResponse::empty("Authenticated"))
}

pub fn delete<T>(target_user: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate_session(auth)?;

    if user.username() == target_user {
        user.delete()?;
    } else if let Ok(admin) = Admin::from_authenticated(user) {
        admin.delete_user(&target_user)?;
    } else {
        return Err(BackendError::PermissionError.into());
    }

    Ok(AuthResponse::empty("Deleted"))
}

pub fn grant_permission<T>(target_user: &str, permission: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    let target_user = User::find_by_name(&target_user)?;

    admin.give_permission(&target_user, &permission)?;

    Ok(AuthResponse::empty("Permission granted"))
}

pub fn revoke_permission<T>(target_user: &str, permission: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    let target_user = User::find_by_name(&target_user)?;

    admin.revoke_permission(&target_user, &permission)?;

    Ok(AuthResponse::empty("Permission revoked"))
}

#[cfg(test)]
mod tests {
    use authentication_backend::Authenticatable;
    use authentication_backend::user_test_helper::teardown_by_name;
    use authentication_backend::test_helper::generate_string;
    use std::panic;
    use super::*;

    #[test]
    fn sign_up_signs_up_new_user() {
        test_wrapper(|username| {
            let auth = Authenticatable::UserAndPass {
                username: username,
                password: "Testp4ss$.",
            };

            let user = sign_up(&auth);

            assert!(user.is_ok(), "Failed to sign in user");
        });
    }

    #[test]
    fn sign_up_with_bad_username_doesnt_sign_up_user() {
        let auth = Authenticatable::UserAndPass {
            username: "",
            password: "Testp4ss$.",
        };

        let user = sign_up(&auth);

        assert!(!user.is_ok(), "Signed up user with empty username");
    }

    #[test]
    fn sign_up_with_bad_password_doesnt_sign_up_user() {
        test_wrapper(|username| {
            let auth = Authenticatable::UserAndPass {
                username: username,
                password: "Testpass$.",
            };

            let user = sign_up(&auth);

            assert!(!user.is_ok(), "Failed to sign in user");
        });
    }

    fn test_wrapper<T>(test: T) -> ()
    where
        T: FnOnce(&str) -> () + panic::UnwindSafe,
    {
        let username = generate_string();
        let result = panic::catch_unwind(|| test(&username));
        teardown_by_name(&username);
        result.unwrap();
    }
}
