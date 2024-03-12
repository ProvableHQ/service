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

mod execute;
pub use execute::*;

pub mod query;
pub use query::*;

pub mod request;
pub use request::*;

pub mod routes;
pub use routes::*;

use snarkvm::circuit::AleoV0;
use snarkvm::ledger::block::Transaction;
use snarkvm::prelude::{
    Authorization, FromBytes, Locator, MainnetV0, Network, Process, StatePath, ToBytes,
};

use anyhow::{anyhow, Result};
use rand_chacha::rand_core::SeedableRng;

pub type CurrentNetwork = MainnetV0;
pub type CurrentAleo = AleoV0;
