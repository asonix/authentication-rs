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

use CONFIG;
use schema::permissions;
use error::Result;
use super::NewPermission;

#[derive(Debug, PartialEq, Queryable, Identifiable, AsChangeset, Associations)]
pub struct Permission {
    id: i32,
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

    pub fn find(permission: &str) -> Result<Self> {
        use diesel::prelude::*;
        use schema::permissions::dsl::*;

        let db = CONFIG.db()?;

        let permission = permissions
            .filter(name.eq(permission))
            .first::<Permission>(db.conn())?;

        Ok(permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helper::generate_string;
    use super::test_helper::teardown;

    #[test]
    fn create_creates_permission() {
        let result = Permission::create(&generate_string());

        assert!(result.is_ok(), "Failed to create permission");

        if let Ok(permission) = result {
            teardown(permission.id);
        }
    }

    #[test]
    fn find_finds_admin_permission() {
        let result = Permission::find("admin");

        assert!(result.is_ok(), "admin permission not found");
    }

    #[test]
    fn find_doesnt_find_fake_permission() {
        let result = Permission::find("This is not a permission");

        assert!(!result.is_ok(), "Fake permission found");
    }
}
