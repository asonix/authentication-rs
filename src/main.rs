#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate authentication;
extern crate rocket;

fn main() {
    rocket::ignite().mount("/", routes![
                           authentication::routes::sign_up,
                           authentication::routes::log_in,
                           authentication::routes::is_authenticated,
                           authentication::routes::verify,
    ]).launch()
}
