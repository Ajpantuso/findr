// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use clap::ValueEnum;
use core::str::FromStr;
use parse_size::parse_size as _parse_size;
use std::os::unix::fs::MetadataExt;
use std::os::unix::prelude::FileTypeExt;
use std::time;

#[derive(Debug, Clone, ValueEnum)]
pub enum TypeFilter {
    #[value(alias = "d")]
    Dir,
    #[value(alias = "x")]
    Executable,
    #[value(alias = "f")]
    File,
    #[value(alias = "p")]
    Pipe,
    #[value(alias = "s")]
    Socket,
    #[value(alias = "l")]
    SymLink,
}

impl TypeFilter {
    pub fn matches(&self, ent: &walkdir::DirEntry) -> bool {
        let ftype = ent.file_type();

        match self {
            Self::Dir => ftype.is_dir(),
            Self::Executable => is_executable::is_executable(ent.path()),
            Self::File => ftype.is_file(),
            Self::Pipe => ftype.is_fifo(),
            Self::Socket => ftype.is_socket(),
            Self::SymLink => ftype.is_symlink(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SizeFilter {
    Less(u64),
    Greater(u64),
    Equal(u64),
}

impl SizeFilter {
    pub fn matches(&self, size: impl PartialOrd<u64>) -> bool {
        match self {
            Self::Equal(u) => size == *u,
            Self::Less(u) => size < *u,
            Self::Greater(u) => size > *u,
        }
    }
}

impl FromStr for SizeFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(maybe_int) = s.strip_prefix('-') {
            Self::Less(parse_size(maybe_int)?)
        } else if let Some(maybe_int) = s.strip_prefix('+') {
            Self::Greater(parse_size(maybe_int)?)
        } else {
            Self::Equal(parse_size(s)?)
        })
    }
}

fn parse_size(s: &str) -> anyhow::Result<u64> {
    _parse_size(s).map_err(|e| anyhow::anyhow!(e))
}

#[derive(Clone, Debug)]
pub enum DurationFilter {
    Less(time::Duration),
    Greater(time::Duration),
}

impl DurationFilter {
    pub fn matches(&self, point_in_time: u64) -> anyhow::Result<bool> {
        let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH)?;
        let point_in_time = time::Duration::from_secs(point_in_time);

        Ok(match self {
            Self::Less(d) => now - *d < point_in_time,
            Self::Greater(d) => now - *d > point_in_time,
        })
    }
}

impl FromStr for DurationFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(maybe_dur) = s.strip_prefix('-') {
            Self::Less(maybe_dur.parse::<humantime::Duration>()?.into())
        } else if let Some(maybe_dur) = s.strip_prefix('+') {
            Self::Greater(maybe_dur.parse::<humantime::Duration>()?.into())
        } else {
            Self::Greater(s.parse::<humantime::Duration>()?.into())
        })
    }
}

#[derive(Clone, Debug)]
pub enum OwnerFilter {
    User(users::User),
    Group(users::Group),
    UserGroup(users::User, users::Group),
}

impl OwnerFilter {
    pub fn matches(&self, ent: &walkdir::DirEntry) -> anyhow::Result<bool> {
        let uid = ent.metadata()?.uid();
        let gid = ent.metadata()?.gid();

        Ok(match self {
            Self::User(u) => u.uid() == uid,
            Self::Group(g) => g.gid() == gid,
            Self::UserGroup(u, g) => u.uid() == uid && g.gid() == gid,
        })
    }
}

impl FromStr for OwnerFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.split_once(':') {
            // "user:"
            Some((user, "")) => Self::User(parse_user(user)?),
            // ":group"
            Some(("", group)) => Self::Group(parse_group(group)?),
            // "user:group"
            Some((user, group)) => Self::UserGroup(parse_user(user)?, parse_group(group)?),
            // "user"
            None => Self::User(parse_user(s)?),
        })
    }
}

fn parse_user(s: &str) -> anyhow::Result<users::User> {
    match u32::from_str(s) {
        Ok(uid) => users::get_user_by_uid(uid),
        Err(_) => users::get_user_by_name(s),
    }
    .ok_or_else(|| anyhow::anyhow!("invalid user '{}'", s))
}

fn parse_group(s: &str) -> anyhow::Result<users::Group> {
    match u32::from_str(s) {
        Ok(gid) => users::get_group_by_gid(gid),
        Err(_) => users::get_group_by_name(s),
    }
    .ok_or_else(|| anyhow::anyhow!("invalid group '{}'", s))
}

#[derive(Clone, Debug)]
pub enum OctalFilter {
    Contains(u32),
    Equal(u32),
    NotContains(u32),
}

impl OctalFilter {
    pub fn matches(&self, o: u32) -> bool {
        match self {
            Self::Contains(m) => *m == (o & *m),
            Self::Equal(m) => *m == (o & 0o777),
            Self::NotContains(m) => 0 == (o & *m),
        }
    }
}

impl FromStr for OctalFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(maybe_octal) = s.strip_prefix('+') {
            Self::Contains(u32::from_str_radix(maybe_octal, 8)?)
        } else if let Some(maybe_octal) = s.strip_prefix('~') {
            Self::NotContains(u32::from_str_radix(maybe_octal, 8)?)
        } else {
            Self::Equal(u32::from_str_radix(s, 8)?)
        })
    }
}
