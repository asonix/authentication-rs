extern crate authentication_backend;

use std::env;
use authentication_backend::models::user::User;
use authentication_backend::models::permission::Permission;
use authentication_backend::models::user_permission::UserPermission;

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        panic!("Argument length must be 2");
    }

    let _executable: Option<String> = args.next();

    let uname: String = args.next().expect("Failed to get username from arguments");

    let user: User = User::find_by_name(&uname).expect(&format!(
        "Unable to find user with username '{}'",
        &uname
    ));
    let permission: Permission =
        Permission::find("admin").expect("Failed to find admin permission");

    let _user_permission = UserPermission::create(&user, &permission).expect(&format!(
        "Failed to make '{}' an admin",
        &uname
    ));
}
