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

pub fn keygen<N: Network>(seed: &str) -> Result<KeygenResponse> {
    let seed = Field::new(<N as Environment>::Field::from_str(seed)?);
    let private_key = PrivateKey::<N>::try_from(seed)?;
    let address = Address::<N>::try_from(&private_key)?;
    Ok(KeygenResponse {
        private_key: private_key.to_bytes_le()?,
        address: address.to_bytes_le()?,
    })
}
