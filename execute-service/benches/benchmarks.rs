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

#[macro_use]
extern crate criterion;

use snarkvm::circuit::AleoV0;
use snarkvm::prelude::{
    Address, Literal, PrivateKey, Process, Value, U64, ToBytes,
};

use execute_service::{CurrentNetwork, execute, ExecuteRequest};

use criterion::{BatchSize, Criterion};
use rand_chacha::rand_core::SeedableRng;
use std::str::FromStr;
use warp::hyper::body::Bytes;


fn bench_execute_transfer_public(c: &mut Criterion) {
    let process = Process::<CurrentNetwork>::load().unwrap();
    let private_key =
        PrivateKey::from_str("APrivateKey1zkpCE9rCw9SixY82xaDrW2Hwxc2f3VjeuR2oZHR81zcuUDV")
            .unwrap();
    let program_id = "credits.aleo";
    let function_name = "transfer_public";
    let inputs = vec![
        Value::<CurrentNetwork>::from(Literal::Address(
            Address::from_str("aleo1zcsyu7wfrdp4n6gq752p3np45sat9d6zun2uhjer2h4skccsgsgq7ndrnj")
                .unwrap(),
        )),
        Value::from(Literal::U64(U64::new(100))),
    ];
    let rng = &mut rand_chacha::ChaCha20Rng::from_entropy();

    let function_authorization = process
        .authorize::<AleoV0, _>(
            &private_key,
            program_id,
            function_name,
            inputs.iter(),
            rng,
        )
        .unwrap();

    // Retrieve the execution ID.
    let execution_id = function_authorization.to_execution_id().unwrap();

    let fee_authorization = process.authorize_fee_public::<AleoV0, _>(
        &private_key,
        100000,
        1000,
        execution_id,
        rng,
    ).unwrap();

    let request = ExecuteRequest {
        function_authorization,
        fee_authorization,
        state_root: None,
        state_path: None,
    };
    let bytes = Bytes::copy_from_slice(&request.to_bytes_le().unwrap());
    c.bench_function("authorize_transfer_public", |b| {
        b.iter_batched(
            || bytes.clone(),
            |bytes| execute(bytes).unwrap(),
            BatchSize::SmallInput,
        )
    });
}
criterion_group! {
    name = routes;
    config = Criterion::default();
    targets = bench_execute_transfer_public,
}
criterion_main!(routes);
