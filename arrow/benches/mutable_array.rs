// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#[macro_use]
extern crate criterion;
use criterion::Criterion;

use rand::Rng;

extern crate arrow;

use arrow::datatypes::Int32Type;
use arrow::util::test_util::seedable_rng;
use arrow::{
    array::*,
    util::bench_util::{create_primitive_array, create_string_array},
};

fn create_slices(size: usize) -> Vec<(usize, usize)> {
    let rng = &mut seedable_rng();

    (0..size)
        .map(|_| {
            let start = rng.random_range(0..size / 2);
            let end = rng.random_range(start + 1..size);
            (start, end)
        })
        .collect()
}

fn bench<T: Array>(v1: &T, slices: &[(usize, usize)]) {
    let data = v1.to_data();
    let mut mutable = MutableArrayData::new(vec![&data], false, 5);
    for (start, end) in slices {
        mutable.try_extend(0, *start, *end).unwrap();
    }
    mutable.freeze();
}

/// Extends only from `v1` (which has no nulls), but `v2` (which has nulls)
/// forces the `MutableArrayData` to track validity, exercising the
/// `use_nulls` branch of `build_extend_null_bits` for null-free sources
fn bench_mixed_no_nulls<T: Array>(v1: &T, v2: &T, slices: &[(usize, usize)]) {
    let d1 = v1.to_data();
    let d2 = v2.to_data();
    let mut mutable = MutableArrayData::new(vec![&d1, &d2], false, 5);
    for (start, end) in slices {
        mutable.try_extend(0, *start, *end).unwrap();
    }
    mutable.freeze();
}

fn add_benchmark(c: &mut Criterion) {
    let v1 = create_string_array::<i32>(1024, 0.0);
    let v2 = create_slices(1024);
    c.bench_function("mutable str 1024", |b| b.iter(|| bench(&v1, &v2)));

    let v1 = create_string_array::<i32>(1024, 0.5);
    let v2 = create_slices(1024);
    c.bench_function("mutable str nulls 1024", |b| b.iter(|| bench(&v1, &v2)));

    let v1 = create_primitive_array::<Int32Type>(1024, 0.0);
    let v2 = create_primitive_array::<Int32Type>(1024, 0.5);
    let slices = create_slices(1024);
    c.bench_function("mutable int32 use_nulls no_null_source 1024", |b| {
        b.iter(|| bench_mixed_no_nulls(&v1, &v2, &slices))
    });
}

criterion_group!(benches, add_benchmark);
criterion_main!(benches);
