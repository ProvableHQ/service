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

pub fn authorize<N: Network>(bytes: Bytes) -> Result<Value> {
    PROCESS.with(|process| {
        // Initialize the process if it is not already initialized.
        if process.borrow().is_none() {
            *process.borrow_mut() = match N::ID {
                MainnetV0::ID => {
                    println!("Loading mainnet process...");
                    Some(ProcessVariant::MainnetV0(Process::load()?))
                }
                TestnetV0::ID => {
                    println!("Loading testnet process...");
                    Some(ProcessVariant::TestnetV0(Process::load()?))
                }
                CanaryV0::ID => {
                    println!("Loading canary process...");
                    Some(ProcessVariant::CanaryV0(Process::load()?))
                }
                _ => panic!("Invalid network"),
            };
        };
        // Compute the `Authorization`.
        process.borrow().as_ref().unwrap().authorize(&bytes)
    })
}
