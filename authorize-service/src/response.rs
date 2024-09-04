// Copyright (C) 2019-2024 Aleo Systems Inc.
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeygenResponse {
    pub private_key: Vec<u8>,
    pub address: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizeResponse<N: Network> {
    #[serde(bound(deserialize = ""))]
    pub function_authorization: Authorization<N>,
    #[serde(bound(deserialize = ""))]
    pub fee_authorization: Authorization<N>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignResponse {
    pub signed_message: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub result: bool,
}
