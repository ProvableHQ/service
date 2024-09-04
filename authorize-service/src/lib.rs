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

pub mod authorize;
pub use authorize::*;

pub mod keygen;
pub use keygen::*;

pub mod process_variant;
pub use process_variant::*;

pub mod request;
pub use request::*;

pub mod response;
pub use response::*;

pub mod routes;
pub use routes::*;

pub mod signature;
pub use signature::*;

use snarkvm::circuit::{Aleo, AleoCanaryV0, AleoTestnetV0, AleoV0};
use snarkvm::prelude::{
    Address, Authorization, CanaryV0, Deserialize, Environment, Field, MainnetV0, Network,
    PrivateKey, Process, Serialize, Signature, TestnetV0, ToBytes,
};

use anyhow::Result;
use rand_chacha::rand_core::SeedableRng;
use serde_json::Value;
use std::cell::RefCell;
use std::str::FromStr;
use warp::hyper::body::Bytes;
