// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::filter::*;
use clap::Parser;
use regex::{self, Regex};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Options {
    #[arg(long = "atime")]
    pub atime_filters: Vec<DurationFilter>,
    #[arg(long = "ctime")]
    pub ctime_filters: Vec<DurationFilter>,
    #[arg(long = "creation-time")]
    pub creation_time_filters: Vec<DurationFilter>,
    #[arg(short = 'p', long = "pattern")]
    pub pattern: Option<Regex>,
    #[arg(default_value = ".")]
    pub dirs: Vec<PathBuf>,
    #[arg(short = 't', long = "type", value_enum)]
    pub type_filters: Vec<TypeFilter>,
    #[arg(long = "max-depth")]
    pub max_depth: Option<usize>,
    #[arg(long = "min-depth")]
    pub min_depth: Option<usize>,
    #[arg(long = "mode")]
    pub mode: Option<OctalFilter>,
    #[arg(long = "mtime")]
    pub mtime_filters: Vec<DurationFilter>,
    #[arg(long = "owner")]
    pub owner: Option<OwnerFilter>,
    #[arg(long = "show-errors")]
    pub show_errors: bool,
    #[arg(short = 's', long = "size")]
    pub size_filters: Vec<SizeFilter>,
}
