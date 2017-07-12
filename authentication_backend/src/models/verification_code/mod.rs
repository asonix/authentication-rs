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

use schema::verification_codes;
use models::user::User;
use error::Result;
use self::new_verification_code::NewVerificationCode;

pub mod new_verification_code;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
pub struct VerificationCode {
    id: i32,
    code: String,
    user_id: i32,
}

impl VerificationCode {
    pub fn create_by_username(username: &str) -> Result<Self> {
        let new_verification_code = NewVerificationCode::new_by_username(username)?;

        new_verification_code.save()
    }

    pub fn create_by_id(user_id: i32) -> Result<Self> {
        let new_verification_code = NewVerificationCode::new_by_id(user_id)?;

        new_verification_code.save()
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn user_id(&self) -> i32 {
        self.user_id
    }
}

#[cfg(test)]
mod tests {}
