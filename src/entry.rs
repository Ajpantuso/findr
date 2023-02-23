// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time;
use walkdir::DirEntry;

pub trait Entry {
    fn path(&self) -> String;
    fn uid(&self) -> Result<u32>;
    fn gid(&self) -> Result<u32>;
    fn atime(&self) -> Result<u64>;
    fn ctime(&self) -> Result<u64>;
    fn created_time(&self) -> Result<u64>;
    fn mtime(&self) -> Result<u64>;
    fn mode(&self) -> Result<u32>;
    fn size(&self) -> Result<u64>;
    fn file_type(&self) -> fs::FileType;
}

pub struct EntryImpl {
    ent: DirEntry,
}

impl From<walkdir::DirEntry> for EntryImpl {
    fn from(ent: walkdir::DirEntry) -> Self {
        Self { ent }
    }
}

impl Entry for EntryImpl {
    fn path(&self) -> String {
        self.ent.path().to_string_lossy().to_string()
    }
    fn uid(&self) -> Result<u32> {
        Ok(self.ent.metadata()?.uid())
    }
    fn gid(&self) -> Result<u32> {
        Ok(self.ent.metadata()?.gid())
    }
    fn atime(&self) -> Result<u64> {
        Ok(self.ent.metadata()?.atime().try_into()?)
    }
    fn ctime(&self) -> Result<u64> {
        Ok(self.ent.metadata()?.ctime().try_into()?)
    }
    fn mtime(&self) -> Result<u64> {
        Ok(self.ent.metadata()?.mtime().try_into()?)
    }
    fn created_time(&self) -> Result<u64> {
        Ok(self
            .ent
            .metadata()?
            .created()?
            .duration_since(time::UNIX_EPOCH)?
            .as_secs())
    }
    fn mode(&self) -> Result<u32> {
        Ok(self.ent.metadata()?.mode())
    }
    fn size(&self) -> Result<u64> {
        Ok(self.ent.metadata()?.size())
    }
    fn file_type(&self) -> fs::FileType {
        self.ent.file_type()
    }
}
