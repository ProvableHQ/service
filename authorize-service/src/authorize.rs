// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Aleo SDK library.

// The Aleo SDK library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo SDK library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo SDK library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

// Initialize a thread-local `ProcessVariant`.
thread_local! {
    pub static PROCESS: RefCell<Option<ProcessVariant>> = const { RefCell::new(None) };
}

pub fn authorize(body: &[u8]) -> Result<Value> {
    PROCESS.with(|process| {
        if let Some(process) = process.borrow().as_ref() {
            process.authorize(body)
        } else {
            unreachable!("The process is always initialized before this function is invoked")
        }
    })
}
