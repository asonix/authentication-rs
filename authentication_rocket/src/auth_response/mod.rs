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

use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket_contrib::Json;
use self::response_body::ResponseBody;

mod response_body;

#[derive(Serialize)]
pub struct AuthResponse {
    message: String,
    #[serde(skip_serializing_if = "ResponseBody::is_empty")]
    data: ResponseBody,
}

impl AuthResponse {
    pub fn new<T>(message: &str, data: T) -> Self
    where
        T: Into<ResponseBody>,
    {
        AuthResponse {
            message: message.to_owned(),
            data: data.into(),
        }
    }

    pub fn empty(message: &str) -> Self {
        AuthResponse {
            message: message.to_owned(),
            data: ResponseBody::NoData,
        }
    }
}

impl<'r> Responder<'r> for AuthResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Json(self).respond_to(req)
    }
}
