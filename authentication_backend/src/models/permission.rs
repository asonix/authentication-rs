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
use schema::permissions;
use error::{Result, Error};
use CONFIG;

#[derive(Queryable, Identifiable, AsChangeset, Associations)]
pub struct Permission {
    id: i32,
    name: String,
}

#[derive(Insertable)]
#[table_name = "permissions"]
pub struct NewPermission {
    name: String,
}

impl Permission {
    pub fn create(name: &str) -> Result<Self> {
        let new_permission = NewPermission::new(name)?;

        new_permission.save()
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl NewPermission {
    fn new(name: &str) -> Result<Self> {
        let name = NewPermission::validate_name(name)?;

        Ok(NewPermission { name: name.to_string() })
    }

    fn save(&self) -> Result<Permission> {
        let db = CONFIG.db()?;

        Ok(diesel::insert(self).into(permissions::table).get_result(
            db.conn(),
        )?)
    }

    fn validate_name(name: &str) -> Result<&str> {
        if name.len() > 0 {
            Ok(name)
        } else {
            Err(Error::InvalidPermissionNameError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_creates_permission() {
        let result = Permission::create(&generate_permission_name());

        assert!(result.is_ok(), "Failed to create permission");

        if let Ok(permission) = result {
            teardown_by_id(permission.id);
        }
    }

    #[test]
    fn new_creates_new_permission() {
        let result = NewPermission::new(&generate_permission_name());

        assert!(result.is_ok(), "Failed to create new_permission");
    }

    #[test]
    fn new_fails_with_bad_name() {
        let result = NewPermission::new("");

        assert!(!result.is_ok(), "Created permission with empty name");
    }

    #[test]
    fn save_saves_new_permission() {
        let new_permission = NewPermission::new(&generate_permission_name()).unwrap();

        let result = new_permission.save();

        assert!(result.is_ok(), "Failed to save new_permission");

        if let Ok(permission) = result {
            teardown_by_id(permission.id);
        }
    }

    #[test]
    fn save_fails_with_duplicate_name() {
        let new_permission = NewPermission::new(&generate_permission_name()).unwrap();

        let permission_one = new_permission.save();
        let permission_two = new_permission.save();

        assert!(permission_one.is_ok(), "Failed to create permission");
        assert!(!permission_two.is_ok(), "Created duplicate permission");

        if let Ok(permission) = permission_one {
            teardown_by_id(permission.id);
        }
    }

    fn teardown_by_id(p_id: i32) -> () {
        use schema::permissions::dsl::*;

        let _ =
            diesel::delete(permissions.filter(id.eq(p_id))).execute(CONFIG.db().unwrap().conn());
    }

    fn generate_permission_name() -> String {
        use rand::Rng;
        use rand::OsRng;

        OsRng::new().unwrap().gen_ascii_chars().take(10).collect()
    }
}
