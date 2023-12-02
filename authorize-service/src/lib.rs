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

pub mod authorize;
pub use authorize::*;

pub mod keygen;
pub use keygen::*;

pub mod request;
pub use request::*;

pub mod response;
pub use response::*;

pub mod routes;
pub use routes::*;

pub mod signature;
pub use signature::*;

use snarkvm::circuit::AleoV0;
use snarkvm::prelude::{
    Address, Authorization, Deserialize, Environment, Field, Network, PrivateKey, Process,
    Serialize, Signature, Testnet3, ToBytes,
};

use anyhow::Result;
use rand_chacha::rand_core::SeedableRng;
use std::cell::RefCell;
use std::str::FromStr;

pub type CurrentNetwork = Testnet3;
pub type CurrentAleo = AleoV0;
