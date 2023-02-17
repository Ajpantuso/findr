// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::filter::*;
use clap::Parser;
use regex::{self, Regex};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(name = clap::crate_name!())]
#[clap(author = clap::crate_authors!())]
#[clap(about = clap::crate_description!())]
pub struct Options {
    /// filters results based on last access time.
    /// By default the value provided filters for
    /// results with an access time further in the past.
    /// Prefixing the value with '-' filters for results
    /// more recent than the value given instead.
    #[arg(long = "atime")]
    pub atime_filters: Vec<DurationFilter>,
    /// filters results based on last changed time.
    /// By default the value provided filters for
    /// results with a change time further in the past.
    /// Prefixing the value with '-' filters for results
    /// more recent than the value given instead.
    #[arg(long = "ctime")]
    pub ctime_filters: Vec<DurationFilter>,
    /// filters results based on creation time.
    /// By default the value provided filters for
    /// results with a creation time further in the past.
    /// Prefixing the value with '-' filters for results
    /// more recent than the value given instead.
    #[arg(long = "creation-time")]
    pub creation_time_filters: Vec<DurationFilter>,
    /// filters results with names matching the given
    /// regular expression.
    #[arg(short = 'p', long = "pattern")]
    pub pattern: Option<Regex>,
    /// specifies the root directories to descend into
    /// when searching.
    #[arg(default_value = ".")]
    pub dirs: Vec<PathBuf>,
    /// filters results matching the given entry types.
    #[arg(short = 't', long = "type", value_enum)]
    pub type_filters: Vec<TypeFilter>,
    /// specifies the maximum level of nested directories
    /// to descend into.
    #[arg(long = "max-depth")]
    pub max_depth: Option<usize>,
    /// specifies the minimum level of nested directories
    /// to descend into.
    #[arg(long = "min-depth")]
    pub min_depth: Option<usize>,
    /// filters results which have the same POSIX permissions
    /// as the value given. Prefixing the given value with
    /// '+' will match all results which have at least the
    /// given permissions. Conversly prefixing with '~'
    /// will match results which do not have the given
    /// permissions.
    #[arg(long = "mode")]
    pub mode: Option<OctalFilter>,
    /// filters results based on modification time.
    /// By default the value provided filters for
    /// results with a modification time further in the past.
    /// Prefixing the value with '-' filters for results
    /// more recent than the value given instead.
    #[arg(long = "mtime")]
    pub mtime_filters: Vec<DurationFilter>,
    /// filters results based on owner:group.
    /// May be specified as "owner", "owner:group"
    /// or ":group" with unspecified owner or group
    /// matching any owner or group respectively.
    #[arg(long = "owner")]
    pub owner: Option<OwnerFilter>,
    /// when enabled outputs any errors encountered
    /// during search. Defaults to 'false'.
    #[arg(long = "show-errors")]
    pub show_errors: bool,
    /// filters results which have size equal to the
    /// given value. Prefixing with '+' returns
    /// results with size greater than the given value
    /// and prefixing with '-' returns those with
    /// size smaller than the given value.
    #[arg(short = 's', long = "size")]
    pub size_filters: Vec<SizeFilter>,
}
