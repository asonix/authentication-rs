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
use error::Result;
use error::Error::InputError;
use error::InputErrorKind::PermissionName;
use super::Permission;
use schema::permissions;

#[derive(Insertable)]
#[table_name = "permissions"]
pub struct NewPermission {
    name: String,
}

impl NewPermission {
    pub fn new(name: &str) -> Result<Self> {
        let name = NewPermission::validate_name(name)?;

        Ok(NewPermission { name: name.to_string() })
    }

    pub fn save(&self) -> Result<Permission> {
        let db = CONFIG.db()?;

        Ok(diesel::insert(self).into(permissions::table).get_result(
            db.conn(),
        )?)
    }

    fn validate_name(name: &str) -> Result<&str> {
        if name.len() > 0 {
            Ok(name)
        } else {
            Err(InputError(PermissionName))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::permission::test_helper::teardown;
    use test_helper::generate_string;

    #[test]
    fn new_creates_new_permission() {
        let result = NewPermission::new(&generate_string());

        assert!(result.is_ok(), "Failed to create new_permission");
    }

    #[test]
    fn new_fails_with_bad_name() {
        let result = NewPermission::new("");

        assert!(!result.is_ok(), "Created permission with empty name");
    }

    #[test]
    fn save_saves_new_permission() {
        let new_permission = NewPermission::new(&generate_string()).unwrap();

        let result = new_permission.save();

        assert!(result.is_ok(), "Failed to save new_permission");

        if let Ok(permission) = result {
            teardown(permission.id);
        }
    }

    #[test]
    fn save_fails_with_duplicate_name() {
        let new_permission = NewPermission::new(&generate_string()).unwrap();

        let permission_one = new_permission.save();
        let permission_two = new_permission.save();

        assert!(permission_one.is_ok(), "Failed to create permission");
        assert!(!permission_two.is_ok(), "Created duplicate permission");

        if let Ok(permission) = permission_one {
            teardown(permission.id);
        }
    }
}
