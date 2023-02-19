// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use clap::ValueEnum;
use is_executable::is_executable;
use std::os::unix::fs::FileTypeExt;
use walkdir::DirEntry;

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
    pub fn matches(&self, ent: &DirEntry) -> bool {
        let ftype = ent.file_type();

        match self {
            Self::Dir => ftype.is_dir(),
            Self::Executable => is_executable(ent.path()),
            Self::File => ftype.is_file(),
            Self::Pipe => ftype.is_fifo(),
            Self::Socket => ftype.is_socket(),
            Self::SymLink => ftype.is_symlink(),
        }
    }
}
