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

use warp::{http::Response, hyper::body::Bytes, Filter, Rejection, Reply};

// POST /execute
pub fn execute_route<N: Network>() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("execute"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(32 * 1024)) // 32 KiB
        .and(warp::body::bytes())
        .and_then(|request_bytes: Bytes| async move {
            let response_bytes = match tokio_rayon::spawn_fifo(|| execute::<N>(request_bytes)).await
            {
                Ok(response_bytes) => response_bytes,
                Err(_) => return Err(warp::reject()),
            };
            let response = match Response::builder()
                .header("content-type", "application/octet-stream")
                .body(response_bytes)
            {
                Ok(response) => response,
                Err(_) => return Err(warp::reject()),
            };
            Ok(response)
        })
}
