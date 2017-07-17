extern crate authentication_backend;

use std::env;
use authentication_backend::{User, UserTrait, VerificationCode};

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        panic!("Argument length must be 2");
    }

    let _executable: Option<String> = args.next();

    let uname: String = args.next().expect("Failed to get username from arguments");

    let mut user: User = User::find_by_name(&uname).expect(&format!(
        "Unable to find user with username '{}'",
        &uname
    ));

    if !user.is_verified() {
        if user.verify() {
            VerificationCode::delete_by_user_id(user.id()).expect(&format!(
                "Failed to delete verification_code for user '{}'",
                user.username(),
                ));
        } else {
            panic!("Failed to verify user '{}'", user.username());
        }
    } else {
        println!("User '{}' is already verified", &uname);
    }
}
