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

pub struct MockMetadata {
    pub uid: u32,
    pub gid: u32,
}

use std::os::unix::fs::MetadataExt;

impl MetadataExt for MockMetadata {
    fn atime(&self) -> i64 {
        0
    }
    fn atime_nsec(&self) -> i64 {
        0
    }
    fn blksize(&self) -> u64 {
        0
    }
    fn blocks(&self) -> u64 {
        0
    }
    fn ctime(&self) -> i64 {
        0
    }
    fn ctime_nsec(&self) -> i64 {
        0
    }
    fn dev(&self) -> u64 {
        0
    }
    fn gid(&self) -> u32 {
        self.gid
    }
    fn ino(&self) -> u64 {
        0
    }
    fn mode(&self) -> u32 {
        0
    }
    fn mtime(&self) -> i64 {
        0
    }
    fn mtime_nsec(&self) -> i64 {
        0
    }
    fn nlink(&self) -> u64 {
        0
    }
    fn rdev(&self) -> u64 {
        0
    }
    fn size(&self) -> u64 {
        0
    }
    fn uid(&self) -> u32 {
        self.uid
    }
}
