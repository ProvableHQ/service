pub mod address;
pub use address::*;

pub mod block;
pub use block::*;

pub mod decoders;
pub use decoders::*;

use snarkvm::prelude::{Address, Block, Header, Input, Network, Transaction, Transition};

use anyhow::{bail, ensure, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;

pub type BondedMapping<N> = HashMap<AddressString<N>, (AddressString<N>, u64)>;
pub type UnbondedMapping<N> = HashMap<AddressString<N>, (u64, u32)>;
pub type WithdrawMapping<N> = HashMap<AddressString<N>, AddressString<N>>;
