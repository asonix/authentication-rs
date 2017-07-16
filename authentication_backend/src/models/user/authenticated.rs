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

use diesel::prelude::*;
use CONFIG;
use error::Result;
use webtoken::Webtoken;
use super::{UserTrait, User, AuthenticatedThisSession};

pub struct Authenticated {
    id: i32,
    username: String,
    verified: bool,
}

impl UserTrait for Authenticated {
    fn id(&self) -> i32 {
        self.id
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn is_verified(&self) -> bool {
        self.verified
    }
}

impl Authenticated {
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        let user = self.fetch_user()?;

        user.verify_password(password)
    }

    pub fn verify(&mut self) -> bool {
        let mut user = match self.fetch_user() {
            Ok(user) => user,
            Err(_) => return false,
        };

        self.verified = user.verify();
        self.verified
    }

    pub fn from_webtoken(webtoken: &str) -> Result<Self> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let (user_id, _) = Webtoken::authenticate(webtoken)?;

        let user = users
            .filter(verified.eq(true))
            .filter(id.eq(user_id))
            .first::<User>(db.conn())?;

        Ok(Authenticated::from_user(&user))
    }

    fn fetch_user(&self) -> Result<User> {
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        let user = users.filter(id.eq(self.id)).first::<User>(db.conn())?;

        Ok(user)
    }

    fn from_user(user: &User) -> Self {
        Authenticated {
            id: UserTrait::id(user),
            username: user.username().to_owned(),
            verified: user.is_verified(),
        }
    }
}

impl From<AuthenticatedThisSession> for Authenticated {
    fn from(session_auth: AuthenticatedThisSession) -> Self {
        Authenticated {
            id: session_auth.id(),
            username: session_auth.username().to_owned(),
            verified: session_auth.is_verified(),
        }
    }
}
