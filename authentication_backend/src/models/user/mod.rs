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

mod user_trait;
mod user;
mod admin;
mod authenticated;
mod authenticated_this_session;
mod new_user;
mod helpers;

#[cfg(feature = "test")]
pub mod test_helper;

pub use self::user_trait::UserTrait;
pub use self::user::User;
pub use self::admin::Admin;
pub use self::authenticated::Authenticated;
pub use self::authenticated_this_session::AuthenticatedThisSession;
pub use self::new_user::NewUser;
