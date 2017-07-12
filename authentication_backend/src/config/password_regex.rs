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

use regex::Regex;

pub struct PasswordRegex {
    numbers: Regex,
    symbols: Regex,
    upper: Regex,
    lower: Regex,
}

impl PasswordRegex {
    pub fn initialize() -> Self {
        PasswordRegex {
            numbers: Regex::new("[0-9]").unwrap(),
            symbols: Regex::new("[!@#$%^&*();\\\\/|<>\"'_+\\-\\.,?=]").unwrap(),
            upper: Regex::new("[A-Z]").unwrap(),
            lower: Regex::new("[a-z]").unwrap(),
        }
    }

    pub fn numbers(&self) -> &Regex {
        &self.numbers
    }

    pub fn symbols(&self) -> &Regex {
        &self.symbols
    }

    pub fn upper(&self) -> &Regex {
        &self.upper
    }

    pub fn lower(&self) -> &Regex {
        &self.lower
    }
}
