// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use std::str::FromStr;

pub fn assert_from_str<T>(s: &str, expected: anyhow::Result<T>)
where
    T: FromStr + PartialEq + Debug,
    <T as FromStr>::Err: Debug,
{
    match expected {
        Ok(f) => assert_eq!(f, T::from_str(s).unwrap()),
        Err(_) => assert!(T::from_str(s).is_err()),
    }
}
