#![feature(plugin, try_trait)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate rocket;
extern crate rocket_contrib;
extern crate authentication_backend;

mod routes;
mod error;
mod auth_result;

pub fn launch() -> () {
    let error = rocket::ignite()
        .mount(
            "/",
            routes![
                routes::sign_up,
                routes::log_in,
                routes::is_authenticated,
                routes::verify,
                routes::delete,
            ],
        )
        .launch();

    panic!("Launch failed! Error: {}", error)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
