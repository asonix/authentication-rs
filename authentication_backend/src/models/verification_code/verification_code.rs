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

use diesel;
use diesel::prelude::*;
use CONFIG;
use schema::verification_codes;
use models::user::User;
use error::Result;
use super::NewVerificationCode;

#[derive(Debug, Queryable, Identifiable, Associations)]
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

    pub fn delete_by_user_id(u_id: i32) -> Result<()> {
        use schema::verification_codes::dsl::{verification_codes, user_id};

        let db = CONFIG.db()?;

        let _ = diesel::delete(verification_codes.filter(user_id.eq(u_id)))
            .execute(db.conn())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::user::UserTrait;
    use models::user::test_helper::with_user;

    #[test]
    fn delete_by_user_id_deletes_verification_code() {
        with_user(|user| {
            let result = VerificationCode::delete_by_user_id(UserTrait::id(&user));

            assert!(result.is_ok(), "Failed to delete verification_code");
        });
    }
}
