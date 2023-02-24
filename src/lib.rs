// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::error;
use std::io::{self, Write};
use std::path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

mod entry;
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
                0 => r.map(entry::EntryImpl::from).map_err(|e| anyhow!(e)),
                u => Err(anyhow!(Error::Terminated(u))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_pattern(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_owner(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_mode(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_type_filters(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_size_filters(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_atime_filters(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_ctime_filters(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_creation_time_filters(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .filter_map(|r| match r {
                Ok(ent) => filter_bool_result(self.matches_mtime_filters(&ent), ent),
                Err(e) if e.is::<anyhow::Error>() => Some(Err(e)),
                Err(e) => Some(Err(anyhow!(e))),
            })
            .try_for_each(|r| -> Result<()> {
                match r {
                    Ok(ent) => print_dirent(&mut out, ent),
                    Err(e) if e.is::<Error>() => Err(e),
                    Err(e) => self.print_error(&mut err, e),
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
    fn matches_owner(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(match &self.options.owner {
            Some(f) => f.matches(ent.uid()?, ent.gid()?),
            None => true,
        })
    }
    fn matches_pattern(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self
            .options
            .pattern
            .as_ref()
            .map_or(true, |p| p.is_match(&ent.path())))
    }
    fn matches_type_filters(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self.options.type_filters.is_empty()
            || self.options.type_filters.iter().any(|t| t.matches(ent)))
    }
    fn matches_atime_filters(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self.options.atime_filters.is_empty() || {
            let atime = ent.atime()?;

            self.options
                .atime_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(atime).map(|b| b && acc))?
        })
    }
    fn matches_ctime_filters(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self.options.ctime_filters.is_empty() || {
            let ctime = ent.ctime()?;

            self.options
                .ctime_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(ctime).map(|b| b && acc))?
        })
    }
    fn matches_creation_time_filters(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self.options.creation_time_filters.is_empty() || {
            let creation_time = ent.created_time()?;

            self.options
                .creation_time_filters
                .iter()
                .try_fold(true, |acc, t| t.matches(creation_time).map(|b| b && acc))?
        })
    }
    fn matches_mode(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(match &self.options.mode {
            Some(f) => f.matches(ent.mode()?),
            None => true,
        })
    }
    fn matches_mtime_filters(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self.options.mtime_filters.is_empty() || {
            let mtime = ent.mtime()?;

            self.options
                .mtime_filters
                .iter()
                .try_fold(true, |acc, f| f.matches(mtime).map(|b| b && acc))?
        })
    }
    fn matches_size_filters(&self, ent: &impl entry::Entry) -> Result<bool> {
        Ok(self.options.size_filters.is_empty() || {
            let size = ent.size()?;

            self.options.size_filters.iter().try_fold(true, |acc, f| {
                Ok::<bool, anyhow::Error>(f.matches(size) && acc)
            })?
        })
    }
    fn print_error(&self, err: &mut impl Write, e: impl AsRef<dyn error::Error>) -> Result<()> {
        if self.options.show_errors {
            writeln!(err, "{}: {}", clap::crate_name!(), e.as_ref())?
        }

        Ok(())
    }
}

fn filter_bool_result<T>(res: Result<bool>, val: T) -> Option<Result<T>> {
    match res {
        Ok(true) => Some(Ok(val)),
        Ok(false) => None,
        Err(e) => Some(Err(e)),
    }
}

fn print_dirent(out: &mut impl Write, ent: impl entry::Entry) -> Result<()> {
    Ok(writeln!(out, "{}", ent.path())?)
}
