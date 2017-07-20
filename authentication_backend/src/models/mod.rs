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

mod user;
mod verification_code;
mod permission;
mod user_permission;

#[cfg(feature = "test")]
pub use self::user::test_helper as user_test_helper;
#[cfg(feature = "test")]
pub use self::user_permission::test_helper as user_permission_test_helper;
#[cfg(feature = "test")]
pub use self::permission::test_helper as permission_test_helper;

pub use self::user::{Admin, Authenticated, AuthenticatedThisSession, User, UserTrait};
pub use self::verification_code::VerificationCode;
pub use self::permission::Permission;
pub use self::user_permission::UserPermission;
