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

use std::cell::RefCell;
use warp::hyper::body::Bytes;

// Initialize a thread-local `ProcessVariant`.
thread_local! {
    pub static PROCESS: RefCell<Option<ProcessVariant>> = const { RefCell::new(None) };
}

pub fn execute<N: Network>(bytes: Bytes) -> Result<Vec<u8>> {
    PROCESS.with(|process| {
        // Initialize the process if it is not already initialized.
        if process.borrow().is_none() {
            *process.borrow_mut() = match N::ID {
                MainnetV0::ID => {
                    Some(ProcessVariant::MainnetV0(load_process::<MainnetV0>()?))
                }
                TestnetV0::ID => {
                    Some(ProcessVariant::TestnetV0(load_process::<TestnetV0>()?))
                }
                CanaryV0::ID => {
                    Some(ProcessVariant::CanaryV0(load_process::<CanaryV0>()?))
                }
                _ => panic!("Invalid network"),
            };
        };
        // Compute the `Execution`.
        process.borrow().as_ref().unwrap().execute(&bytes)
    })
}

/// A helper function to load a Process and the necessary proving keys.
pub fn load_process<N: Network>() -> Result<Process<N>> {
    // Load the process.
    let process = Process::load()?;
    // Initialize the proving keys for the functions in credits.aleo.
    let credits_program = process.get_program("credits.aleo")?;
    for (function_name, _) in credits_program.functions() {
        // Get the proving key. This method will load the proving key if it does not exist.
        let _ = process.get_proving_key("credits.aleo", function_name)?;
    }
    Ok(process)
}
