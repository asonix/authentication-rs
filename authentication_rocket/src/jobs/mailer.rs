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

use super::{Result, BackgroundError};
use authentication_backend::{VerificationCode, UserTrait, User};

fn handle_code(user_id: i32) -> Result {
    let vc = match VerificationCode::find_by_user_id(user_id) {
        Ok(vc) => vc,
        Err(_) => {
            return Err(BackgroundError::ProcessingError(
                "Could not find verification_code".to_owned(),
            ))
        }
    };

    let user = match User::find_by_id(user_id) {
        Ok(user) => user,
        Err(_) => {
            return Err(BackgroundError::ProcessingError(
                "Could not find user".to_owned(),
            ))
        }
    };

    println!(
        "Sending email to user '{}' with verification code '{}'",
        user.username(),
        vc.code()
    );

    Ok(())
}

pub fn verification_code(msg: &Option<i32>) -> Result {
    match *msg {
        Some(msg) => handle_code(msg),
        None => Ok(()),
    }
}
