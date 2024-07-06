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

/// Sign a message with a private key
///
/// @param {string} private_key Private key to sign the message with
/// @param {Uint8Array} message Byte representation of the message to sign
/// @returns {SignatureResponse} Signed message in little endian archive
pub fn sign<N: Network>(request: SignRequest<N>) -> Result<SignResponse> {
    let signature = Signature::<N>::sign_bytes(
        &request.private_key,
        &request.message,
        &mut rand_chacha::ChaCha20Rng::from_entropy(),
    )?;

    Ok(SignResponse {
        signed_message: signature.to_bytes_le()?,
    })
}

/// Verify a signature of a message with an address
///
/// @param {VerifyRequest} verify_request Request containing the address, message, and signature
/// @returns {VerifyResponse} True if the signature is valid, false otherwise
pub fn verify<N: Network>(request: VerifyRequest<N>) -> Result<VerifyResponse> {
    Ok(VerifyResponse {
        result: request
            .signature
            .verify_bytes(&request.address, &request.message),
    })
}
