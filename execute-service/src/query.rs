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

use async_trait::async_trait;

use snarkvm::ledger::query::QueryTrait;
use snarkvm::prelude::Field;

#[derive(Clone, Debug)]
pub struct StaticQuery<N: Network> {
    pub state_root: Option<N::StateRoot>,
    pub state_path: Option<StatePath<N>>,
}

impl<N: Network> StaticQuery<N> {
    pub fn new(state_root: Option<N::StateRoot>, state_path: Option<StatePath<N>>) -> Self {
        Self {
            state_root,
            state_path,
        }
    }
}

#[async_trait(?Send)]
impl<N: Network> QueryTrait<N> for StaticQuery<N> {
    fn current_state_root(&self) -> Result<N::StateRoot> {
        self.state_root
            .ok_or_else(|| anyhow!("State root is not set."))
    }

    async fn current_state_root_async(&self) -> Result<N::StateRoot> {
        self.state_root
            .ok_or_else(|| anyhow!("State root is not set."))
    }

    fn get_state_path_for_commitment(&self, _: &Field<N>) -> Result<StatePath<N>> {
        self.state_path
            .clone()
            .ok_or_else(|| anyhow!("State path is not set."))
    }

    async fn get_state_path_for_commitment_async(&self, _: &Field<N>) -> Result<StatePath<N>> {
        self.state_path
            .clone()
            .ok_or_else(|| anyhow!("State path is not set."))
    }
}
