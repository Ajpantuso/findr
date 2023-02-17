// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use findr::{self, options};
use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::io;
use std::process;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

const SIG_EXIT_MARKER: usize = 128;

fn main() -> io::Result<()> {
    let term_sig = Arc::new(AtomicUsize::new(0));

    signal_flag::register_usize(SIGTERM, Arc::clone(&term_sig), SIGTERM as usize)?;
    signal_flag::register_usize(SIGINT, Arc::clone(&term_sig), SIGINT as usize)?;

    let code: i32 = match findr::Command::new(&options::Options::parse()).run(term_sig) {
        Ok(()) => 0,
        Err(e) => match e.downcast::<findr::Error>() {
            Ok(e) => match e {
                findr::Error::Terminated(u) => (u + SIG_EXIT_MARKER).try_into().unwrap(),
            },
            Err(e) => {
                eprintln!("{e}");

                1
            }
        },
    };

    process::exit(code)
}
