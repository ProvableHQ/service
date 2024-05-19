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
    static PROCESS: RefCell<Option<ProcessVariant>> = RefCell::new(None);
}

pub fn authorize<A: Aleo<Network = N>, N: Network>(request: AuthorizeRequest<N>) -> Result<AuthorizeResponse<N>> {
    PROCESS.with(|process| {
        if let Some(process) = *process.borrow() {
            // Initialize the RNG.
            let rng = &mut rand_chacha::ChaCha20Rng::from_entropy();

            // Authorize the function.
            let function_authorization = process.authorize::<A, _>(
                &request.private_key,
                request.program_id,
                request.function_name,
                request.inputs.iter(),
                rng,
            )?;

            // Get the execution ID.
            let execution_id = function_authorization.to_execution_id()?;

            // Authorize the fee.
            let fee_authorization = process.authorize_fee_public::<A, _>(
                &request.private_key,
                *request.base_fee_in_microcredits,
                *request.priority_fee_in_microcredits,
                execution_id,
                rng,
            )?;

            // Construct the response.
            let response = AuthorizeResponse {
                function_authorization,
                fee_authorization,
            };

            // Return the response.
            Ok(response)
        } else {
            unreachable!("The process is always initialized before this function is invoked")
        }

    })
}
