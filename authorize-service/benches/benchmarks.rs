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
    Address, FromBytes, Identifier, Literal, MainnetV0, PrivateKey, Process, ProgramID, Signature,
    Value, U64,
};

use authorize_service::{keygen, sign, verify, AuthorizeRequest, SignRequest, VerifyRequest};

use criterion::{BatchSize, Criterion};
use rand_chacha::rand_core::SeedableRng;
use std::str::FromStr;

type CurrentNetwork = MainnetV0;

fn bench_private_key_from_seed(c: &mut Criterion) {
    c.bench_function("private_key_from_seed", |b| {
        b.iter(|| keygen::<CurrentNetwork>("94030298402398402"))
    });
}

fn bench_authorize_transfer_public(c: &mut Criterion) {
    let request = AuthorizeRequest::<CurrentNetwork> {
        private_key: PrivateKey::from_str(
            "APrivateKey1zkpCE9rCw9SixY82xaDrW2Hwxc2f3VjeuR2oZHR81zcuUDV",
        )
        .unwrap(),
        program_id: ProgramID::from_str("credits.aleo").unwrap(),
        function_name: Identifier::from_str("transfer_public").unwrap(),
        inputs: vec![
            Value::from_str("aleo1zcsyu7wfrdp4n6gq752p3np45sat9d6zun2uhjer2h4skccsgsgq7ndrnj")
                .unwrap(),
            Value::from_str("100u64").unwrap(),
        ],
        base_fee_in_microcredits: U64::new(300000),
        priority_fee_in_microcredits: U64::new(0),
    };
    let body = serde_json::to_vec(&request).unwrap();
    c.bench_function("authorize_transfer_public", |b| {
        b.iter_batched(
            || body.clone(),
            |body| authorize_service::authorize(&body).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

fn bench_authorize(c: &mut Criterion) {
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

    c.bench_function("general_authorize", move |b| {
        b.iter_batched(
            || inputs.clone(),
            |inputs| {
                process
                    .authorize::<AleoV0, _>(
                        &private_key,
                        program_id,
                        function_name,
                        inputs.iter(),
                        rng,
                    )
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_sign(c: &mut Criterion) {
    let request = SignRequest::<CurrentNetwork> {
        private_key: PrivateKey::from_str(
            "APrivateKey1zkpCE9rCw9SixY82xaDrW2Hwxc2f3VjeuR2oZHR81zcuUDV",
        )
        .unwrap(),
        message: "Hello, world!".as_bytes().to_vec(),
    };

    c.bench_function("sign", move |b| {
        b.iter_batched(
            || request.clone(),
            |request| sign(request).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

fn bench_verify(c: &mut Criterion) {
    // Sign a message.
    let sign_request = SignRequest::<CurrentNetwork> {
        private_key: PrivateKey::from_str(
            "APrivateKey1zkpCE9rCw9SixY82xaDrW2Hwxc2f3VjeuR2oZHR81zcuUDV",
        )
        .unwrap(),
        message: "Hello, world!".as_bytes().to_vec(),
    };
    let sign_response = sign(sign_request).unwrap();

    // Verify the signature matches the address.
    let verify_request = VerifyRequest {
        address: Address::from_str(
            "aleo1zcsyu7wfrdp4n6gq752p3np45sat9d6zun2uhjer2h4skccsgsgq7ndrnj",
        )
        .unwrap(),
        signature: Signature::<CurrentNetwork>::from_bytes_le(&sign_response.signed_message)
            .unwrap(),
        message: "Hello, world!".as_bytes().to_vec(),
    };

    c.bench_function("verify", move |b| {
        b.iter_batched(
            || (verify_request.clone()),
            |verify_request| verify(verify_request).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group! {
    name = routes;
    config = Criterion::default();
    targets = bench_private_key_from_seed, bench_authorize_transfer_public, bench_authorize, bench_sign, bench_verify
}
criterion_main!(routes);
