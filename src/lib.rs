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
            .filter_map(curry_filter(|e| self.matches_pattern(e)))
            .filter_map(curry_filter(|e| self.matches_owner(e)))
            .filter_map(curry_filter(|e| self.matches_mode(e)))
            .filter_map(curry_filter(|e| self.matches_type_filters(e)))
            .filter_map(curry_filter(|e| self.matches_size_filters(e)))
            .filter_map(curry_filter(|e| self.matches_atime_filters(e)))
            .filter_map(curry_filter(|e| self.matches_ctime_filters(e)))
            .filter_map(curry_filter(|e| self.matches_creation_time_filters(e)))
            .filter_map(curry_filter(|e| self.matches_mtime_filters(e)))
            .try_for_each(|r| -> Result<()> {
                match r {
                    Ok(ent) => print_dirent(&mut out, ent),
                    Err(e) if e.is::<Error>() => Err(e),
                    Err(e) => self.print_error(&mut err, e),
                }
            })
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
    fn matches_owner<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok(match &self.options.owner {
            Some(f) => f.matches(ent.uid()?, ent.gid()?).then_some(ent),
            None => Some(ent),
        })
    }
    fn matches_pattern<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok(match &self.options.pattern {
            Some(p) => p.is_match(&ent.path()).then_some(ent),
            None => Some(ent),
        })
    }
    fn matches_type_filters<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok((self.options.type_filters.is_empty()
            || self.options.type_filters.iter().any(|t| t.matches(&ent)))
        .then_some(ent))
    }
    fn matches_atime_filters<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok((self.options.atime_filters.is_empty() || {
            let atime = ent.atime()?;

            self.options
                .atime_filters
                .iter()
                .map(|f| f.matches(atime))
                .try_all()?
        })
        .then_some(ent))
    }
    fn matches_ctime_filters<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok((self.options.ctime_filters.is_empty() || {
            let ctime = ent.ctime()?;

            self.options
                .ctime_filters
                .iter()
                .map(|f| f.matches(ctime))
                .try_all()?
        })
        .then_some(ent))
    }
    fn matches_creation_time_filters<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok((self.options.creation_time_filters.is_empty() || {
            let creation_time = ent.created_time()?;

            self.options
                .creation_time_filters
                .iter()
                .map(|f| f.matches(creation_time))
                .try_all()?
        })
        .then_some(ent))
    }
    fn matches_mode<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok(match &self.options.mode {
            Some(f) => f.matches(ent.mode()?).then_some(ent),
            None => Some(ent),
        })
    }
    fn matches_mtime_filters<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok((self.options.mtime_filters.is_empty() || {
            let mtime = ent.mtime()?;

            self.options
                .mtime_filters
                .iter()
                .map(|f| f.matches(mtime))
                .try_all()?
        })
        .then_some(ent))
    }
    fn matches_size_filters<E: entry::Entry>(&self, ent: E) -> Result<Option<E>> {
        Ok((self.options.size_filters.is_empty() || {
            let size = ent.size()?;

            self.options.size_filters.iter().all(|f| f.matches(size))
        })
        .then_some(ent))
    }
    fn print_error(&self, err: &mut impl Write, e: impl AsRef<dyn error::Error>) -> Result<()> {
        if self.options.show_errors {
            writeln!(err, "{}: {}", clap::crate_name!(), e.as_ref())?
        }

        Ok(())
    }
}

fn curry_filter<E: entry::Entry>(
    f: impl Fn(E) -> Result<Option<E>>,
) -> impl Fn(Result<E>) -> Option<Result<E>> {
    move |r| match r {
        Ok(ent) => f(ent).transpose(),
        Err(e) => Some(Err(e)),
    }
}

fn print_dirent(out: &mut impl Write, ent: impl entry::Entry) -> Result<()> {
    Ok(writeln!(out, "{}", ent.path())?)
}

trait TryBoolExt {
    fn try_all(&mut self) -> Result<bool>;
}

impl<I: Iterator<Item = Result<bool>>> TryBoolExt for I {
    fn try_all(&mut self) -> Result<bool> {
        for b in self {
            if !b? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
