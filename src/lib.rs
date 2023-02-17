// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::error;
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time;
use walkdir::WalkDir;

mod filter;
pub mod options;

pub struct Command<'a> {
    options: &'a options::Options,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("terminated")]
    Terminated(usize),
}

impl<'a> Command<'a> {
    pub fn new(options: &'a options::Options) -> Self {
        Self { options }
    }
    pub fn run(&self, term_sig: Arc<AtomicUsize>) -> Result<()> {
        let mut out = io::stdout().lock();
        let mut err = io::stderr().lock();

        self.options
            .dirs
            .iter()
            .flat_map(|p| self.new_walker(p))
            .map(|r| match term_sig.load(Ordering::Relaxed) {
                0 => r.map_err(|e| anyhow!(e)),
                u => Err(anyhow!(Error::Terminated(u))),
            })
            .filter_map(apply_filter(self, Command::matches_pattern))
            .filter_map(apply_filter(self, Command::matches_owner))
            .filter_map(apply_filter(self, Command::matches_mode))
            .filter_map(apply_filter(self, Command::matches_type_filters))
            .filter_map(apply_filter(self, Command::matches_size_filters))
            .filter_map(apply_filter(self, Command::matches_atime_filters))
            .filter_map(apply_filter(self, Command::matches_ctime_filters))
            .filter_map(apply_filter(self, Command::matches_creation_time_filters))
            .filter_map(apply_filter(self, Command::matches_mtime_filters))
            .try_for_each(|r| -> Result<()> {
                match r {
                    Ok(ent) => print_dirent(&mut out, ent),
                    Err(e) => {
                        if e.is::<Error>() {
                            return Err(e);
                        }

                        self.print_error(&mut err, e)
                    }
                }
            })?;

        Ok(())
    }
    fn new_walker(&self, path: impl AsRef<path::Path>) -> walkdir::WalkDir {
        let mut walker = WalkDir::new(path);

        if let Some(depth) = self.options.min_depth {
            walker = walker.min_depth(depth);
        }
        if let Some(depth) = self.options.max_depth {
            walker = walker.max_depth(depth);
        }

        walker
    }
    fn matches_owner(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        self.options
            .owner
            .as_ref()
            .map_or(Ok(true), |o| o.matches(ent))
    }
    fn matches_pattern(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        Ok(self
            .options
            .pattern
            .as_ref()
            .map_or(true, |p| p.is_match(&ent.path().to_string_lossy())))
    }
    fn matches_type_filters(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        Ok(self.options.type_filters.is_empty()
            || self.options.type_filters.iter().any(|t| t.matches(ent)))
    }
    fn matches_atime_filters(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        let atime = ent.metadata()?.atime().try_into()?;

        Ok(self.options.atime_filters.is_empty()
            || self
                .options
                .atime_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(atime).map(|b| b && acc))?)
    }
    fn matches_ctime_filters(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        let ctime = ent.metadata()?.ctime().try_into()?;

        Ok(self.options.ctime_filters.is_empty()
            || self
                .options
                .ctime_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(ctime).map(|b| b && acc))?)
    }
    fn matches_creation_time_filters(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        let creation_time = ent
            .metadata()?
            .created()?
            .duration_since(time::UNIX_EPOCH)?
            .as_secs();

        Ok(self.options.creation_time_filters.is_empty()
            || self
                .options
                .creation_time_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(creation_time).map(|b| b && acc))?)
    }
    fn matches_mode(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        let mode = ent.metadata()?.mode();

        Ok(self.options.mode.as_ref().map_or(true, |m| m.matches(mode)))
    }
    fn matches_mtime_filters(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        let mtime = ent.metadata()?.mtime().try_into()?;

        Ok(self.options.mtime_filters.is_empty()
            || self
                .options
                .mtime_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(mtime).map(|b| b && acc))?)
    }
    fn matches_size_filters(&self, ent: &walkdir::DirEntry) -> Result<bool> {
        let size = ent.metadata()?.size();

        Ok(self.options.size_filters.is_empty()
            || self.options.size_filters.iter().all(|s| s.matches(size)))
    }
    fn print_error(&self, err: &mut impl Write, e: impl AsRef<dyn error::Error>) -> Result<()> {
        if self.options.show_errors {
            writeln!(err, "{}: {}", clap::crate_name!(), e.as_ref())?
        }

        Ok(())
    }
}

fn apply_filter<'a>(
    cmd: &'a Command<'a>,
    f: impl Fn(&'a Command<'a>, &walkdir::DirEntry) -> Result<bool> + 'a,
) -> impl Fn(Result<walkdir::DirEntry>) -> Option<Result<walkdir::DirEntry>> + 'a {
    move |res: Result<walkdir::DirEntry>| match res {
        Ok(ent) => match f(cmd, &ent) {
            Ok(true) => Some(Ok(ent)),
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        },
        Err(e) => Some(Err(anyhow!(e))),
    }
}

fn print_dirent(out: &mut impl Write, ent: walkdir::DirEntry) -> Result<()> {
    Ok(writeln!(out, "{}", ent.path().to_string_lossy())?)
}
