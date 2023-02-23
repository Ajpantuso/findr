// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum OwnerFilter {
    User(u32),
    Group(u32),
    UserGroup(u32, u32),
}

impl OwnerFilter {
    pub fn matches(&self, uid: u32, gid: u32) -> bool {
        match self {
            Self::User(u) => *u == uid,
            Self::Group(g) => *g == gid,
            Self::UserGroup(u, g) => *u == uid && *g == gid,
        }
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

fn parse_user(s: &str) -> anyhow::Result<u32> {
    match u32::from_str(s) {
        Ok(uid) => users::get_user_by_uid(uid),
        Err(_) => users::get_user_by_name(s),
    }
    .map(|u| u.uid())
    .ok_or_else(|| anyhow::anyhow!("invalid user '{}'", s))
}

fn parse_group(s: &str) -> anyhow::Result<u32> {
    match u32::from_str(s) {
        Ok(gid) => users::get_group_by_gid(gid),
        Err(_) => users::get_group_by_name(s),
    }
    .map(|g| g.gid())
    .ok_or_else(|| anyhow::anyhow!("invalid group '{}'", s))
}

#[cfg(test)]
mod tests {
    use super::OwnerFilter;
    use crate::filter::testing::*;
    use anyhow::{anyhow, Result};
    use test_case::test_case;
    use users::{get_current_gid, get_current_groupname, get_current_uid, get_current_username};

    #[test_case(
        &current_user_name(),
        Ok(OwnerFilter::User(get_current_uid()))
        ; "current user"
    )]
    #[test_case(
        &format!("{}:", current_user_name()),
        Ok(OwnerFilter::User(get_current_uid()))
        ; "current user ':' suffix"
    )]
    #[test_case(
        &format!(":{}", current_group_name()),
        Ok(OwnerFilter::Group(get_current_gid()))
        ; "current group"
    )]
    #[test_case(
        &format!("{}:{}", current_user_name(), current_group_name()),
        Ok(OwnerFilter::UserGroup(get_current_uid(), get_current_gid()))
        ; "current user:group"
    )]
    #[test_case("dne_user", Err(anyhow!("")) ; "non-existent user")]
    #[test_case(":dne_group", Err(anyhow!("")) ; "non-existent group")]
    fn from_str(s: &str, expected: Result<OwnerFilter>) {
        assert_from_str(s, expected)
    }
    #[test_case(
        OwnerFilter::User(get_current_uid()),
        get_current_uid(), get_current_gid(),
        true
        ; "current user matches"
    )]
    #[test_case(
        OwnerFilter::Group(get_current_gid()),
        get_current_uid(), get_current_gid(),
        true
        ; "current group matches"
    )]
    #[test_case(
        OwnerFilter::UserGroup(get_current_uid(), get_current_gid()),
        get_current_uid(), get_current_gid(),
        true
        ; "current user and group matches"
    )]
    #[test_case(
        OwnerFilter::User(get_current_uid()),
        get_current_uid()+1, get_current_gid()+1,
        false
        ; "current user does not match current uid + 1"
    )]
    #[test_case(
        OwnerFilter::Group(get_current_gid()),
        get_current_uid()+1, get_current_gid()+1,
        false
        ; "current group does not match current gid + 1"
    )]
    #[test_case(
        OwnerFilter::UserGroup(get_current_uid(), get_current_gid()),
        get_current_uid()+1, get_current_gid()+1,
        false
        ; "current user and group does not match current uid +1 and gid + 1"
    )]
    fn matches(f: OwnerFilter, uid: u32, gid: u32, expected: bool) {
        assert_eq!(expected, f.matches(uid, gid))
    }
    fn current_user_name() -> String {
        get_current_username()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
    fn current_group_name() -> String {
        get_current_groupname()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
}
