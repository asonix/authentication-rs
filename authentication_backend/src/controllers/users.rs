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

use authentication_background::{Message, MsgSender};
use authenticatable::ToAuth;
use webtoken::Webtoken;
use models::{Admin, Authenticated, User, UserTrait};
use error::{Result, Error};

pub fn sign_up<T>(auth: &T, sender: &MsgSender<i32>) -> Result<User>
where
    T: ToAuth,
{
    let user = User::create(auth)?;

    match sender.send(Message::new("mail", Some(user.id()))) {
        _ => (),
    };

    Ok(user)
}

pub fn log_in<T>(auth: &T) -> Result<Webtoken>
where
    T: ToAuth,
{
    let user = User::authenticate_session(auth)?;

    user.create_webtoken()
}

pub fn is_authenticated<T>(auth: &T) -> Result<Authenticated>
where
    T: ToAuth,
{
    User::authenticate(auth)
}

pub fn delete<T>(target_user: &str, auth: &T) -> Result<()>
where
    T: ToAuth,
{
    let user = User::authenticate_session(auth)?;

    if user.username() == target_user {
        user.delete()?;
    } else if let Ok(admin) = Admin::from_authenticated(user) {
        admin.delete_user(target_user)?;
    } else {
        return Err(Error::PermissionError);
    }

    Ok(())
}

pub fn grant_permission<T>(target_user: &str, permission: &str, auth: &T) -> Result<()>
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    let target_user = User::find_by_name(target_user)?;

    admin.give_permission(&target_user, permission)?;

    Ok(())
}

pub fn revoke_permission<T>(target_user: &str, permission: &str, auth: &T) -> Result<()>
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    let target_user = User::find_by_name(target_user)?;

    admin.revoke_permission(&target_user, permission)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use authentication_background::Message;
    use std::sync::mpsc;
    use std::panic;
    use user_test_helper::{teardown_by_name, with_user, with_auth_session, with_admin};
    use authenticatable::Authenticatable;
    use test_helper::{generate_string, test_password};
    use super::*;

    #[test]
    fn sign_up_signs_up_new_user() {
        test_wrapper(|username| {
            with_msg_sender(1, |sender| {
                let auth = Authenticatable::UserAndPass {
                    username: username,
                    password: "Testp4ss$.",
                };

                let user = sign_up(&auth, &sender);

                assert!(user.is_ok(), "Failed to sign in user");
            });
        });
    }

    #[test]
    fn sign_up_with_bad_username_doesnt_sign_up_user() {
        with_msg_sender(0, |sender| {
            let auth = Authenticatable::UserAndPass {
                username: "",
                password: "Testp4ss$.",
            };

            let user = sign_up(&auth, &sender);

            assert!(!user.is_ok(), "Signed up user with empty username");
        });
    }

    #[test]
    fn sign_up_with_bad_password_doesnt_sign_up_user() {
        test_wrapper(|username| {
            with_msg_sender(0, |sender| {
                let auth = Authenticatable::UserAndPass {
                    username: username,
                    password: "This is a bad password",
                };

                let user = sign_up(&auth, &sender);

                assert!(!user.is_ok(), "Failed to sign in user");
            });
        });
    }

    #[test]
    fn log_in_logs_in() {
        with_user(|mut user| {
            assert!(user.verify(), "Failed to verify user");

            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let user = log_in(&auth);

            assert!(user.is_ok(), "Failed to log in user");
        });
    }

    #[test]
    fn log_in_fails_with_bad_password() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: "This is not the password",
            };

            let user = log_in(&auth);

            assert!(!user.is_ok(), "Failed to log in user");
        });
    }

    #[test]
    fn is_authenticated_works() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let result = is_authenticated(&auth);

            assert!(result.is_ok(), "Failed to verify authentication");
        });
    }

    #[test]
    fn is_authenticated_fails_with_bad_user() {
        let auth = Authenticatable::UserAndPass {
            username: "not real",
            password: "obviously fake",
        };

        let result = is_authenticated(&auth);

        assert!(
            !result.is_ok(),
            "Fake user should not have been authenticated"
        );
    }

    #[test]
    fn is_authenticated_works_with_token() {
        with_auth_session(|mut auth| {
            auth.verify();
            let auth = auth;
            let token = auth.create_webtoken().expect("Failed to create webtoken");

            let auth = Authenticatable::UserToken { user_token: token.user_token() };

            let result = is_authenticated(&auth);

            assert!(result.is_ok(), "Failed to verify authentication");
        });
    }

    #[test]
    fn is_authenticated_works_with_username_and_token() {
        with_auth_session(|mut auth| {
            auth.verify();
            let auth = auth;
            let token = auth.create_webtoken().expect("Failed to create webtoken");

            let auth = Authenticatable::UserTokenAndPass {
                user_token: token.user_token(),
                password: test_password(),
            };

            let result = is_authenticated(&auth);

            assert!(result.is_ok(), "Failed to verify authentication");
        });
    }

    #[test]
    fn delete_with_admin_deletes_user() {
        with_admin(|admin| {
            with_user(|user| {
                let auth = Authenticatable::UserAndPass {
                    username: admin.username(),
                    password: test_password(),
                };

                let result = delete(user.username(), &auth);

                assert!(result.is_ok(), "Failed to delete user");
            });
        });
    }

    #[test]
    fn delete_with_user_fails_to_delete_user() {
        with_user(|user| {
            with_user(|user2| {
                let auth = Authenticatable::UserAndPass {
                    username: user.username(),
                    password: test_password(),
                };

                let result = delete(user2.username(), &auth);

                assert!(!result.is_ok(), "Deleted user with bad permissions");
            });
        });
    }

    #[test]
    fn delete_with_user_deletes_self() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let result = delete(user.username(), &auth);

            assert!(result.is_ok(), "User should be alowed to delete self");
        });
    }

    #[test]
    fn grant_permission_grants_permission() {
        with_admin(|admin| {
            with_user(|user| {
                let auth = Authenticatable::UserAndPass {
                    username: admin.username(),
                    password: test_password(),
                };

                let result = grant_permission(user.username(), "admin", &auth);

                assert!(result.is_ok(), "Admin failed to grant User Permission");
            });
        });
    }

    #[test]
    fn user_cannot_grant_permissions() {
        with_user(|user| {
            with_user(|user2| {
                let auth = Authenticatable::UserAndPass {
                    username: user.username(),
                    password: test_password(),
                };

                let result = grant_permission(user2.username(), "admin", &auth);

                assert!(!result.is_ok(), "Non-Admin User granted permission");
            });
        });
    }

    #[test]
    fn user_cannot_grant_self_permissions() {
        with_user(|user| {
            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let result = grant_permission(user.username(), "admin", &auth);

            assert!(!result.is_ok(), "Non-Admin User granted permission");
        });
    }

    #[test]
    fn admin_can_revoke_permission() {
        with_admin(|admin| {
            with_admin(|admin2| {
                let auth = Authenticatable::UserAndPass {
                    username: admin.username(),
                    password: test_password(),
                };

                let result = revoke_permission(admin2.username(), "admin", &auth);

                assert!(result.is_ok(), "Failed to revoke permission");
            });
        });
    }

    #[test]
    fn user_cannot_revoke_permission() {
        with_user(|user| {
            with_admin(|admin| {
                let auth = Authenticatable::UserAndPass {
                    username: user.username(),
                    password: test_password(),
                };

                let result = revoke_permission(admin.username(), "admin", &auth);

                assert!(!result.is_ok(), "Non-Admin User revoked permission");
            });
        });
    }

    fn with_msg_sender<T>(sent: usize, test: T) -> ()
    where
        T: FnOnce(MsgSender<i32>) -> () + panic::UnwindSafe,
    {
        let (sender, receiver) = mpsc::channel::<Message<i32>>();

        test(sender); // consume sender

        let len = receiver.iter().collect::<Vec<_>>().len();

        assert_eq!(
            len,
            sent,
            "Did not send correct number of messages, expected: {}, sent: {}",
            sent,
            len,
        );
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
