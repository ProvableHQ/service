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

use snarkvm::prelude::{Identifier, ProgramID, Value, U64};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizeRequest<N: Network> {
    #[serde(bound(deserialize = ""))]
    pub private_key: PrivateKey<N>,
    #[serde(bound(deserialize = ""))]
    pub program_id: ProgramID<N>,
    #[serde(bound(deserialize = ""))]
    pub function_name: Identifier<N>,
    #[serde(bound(deserialize = ""))]
    pub inputs: Vec<Value<N>>,
    #[serde(bound(deserialize = ""))]
    pub base_fee_in_microcredits: U64<N>,
    #[serde(bound(deserialize = ""))]
    pub priority_fee_in_microcredits: U64<N>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignRequest<N: Network> {
    #[serde(bound(deserialize = ""))]
    pub private_key: PrivateKey<N>,
    #[serde(bound(deserialize = ""))]
    pub message: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyRequest<N: Network> {
    #[serde(bound(deserialize = ""))]
    pub address: Address<N>,
    pub message: Vec<u8>,
    #[serde(bound(deserialize = ""))]
    pub signature: Signature<N>,
}
