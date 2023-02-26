// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod findr {
    use anyhow::Result;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use tempfile;
    use test_case::test_case;

    #[test_case(&["--pattern=.+.md"], r"\./one/b.md" ; ".md files")]
    #[test_case(&["--type=x"], r"\./a.txt" ; "executables")]
    #[test_case(&["--type=l"], r"\./three/d.txt" ; "symlink")]
    #[test_case(&["--mode=444"], r"\./one/two/c.txt" ; "readonly")]
    #[test_case(&["--size=8"], r"\./one/b.md" ; "size == 8")]
    fn valid(args: &[&str], expected: &str) -> Result<()> {
        let dir = setup_root_dir()?;

        Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .current_dir(dir.path())
            .args(args)
            .assert()
            .stdout(predicate::str::is_match(expected).unwrap())
            .success();

        Ok(dir.close()?)
    }

    #[test_case(&["dne"], "IO error for operation on dne: No such file or directory" ; "non-existent root directory")]
    #[test_case(&["--pattern", "["], "regex parse error" ; "invalid pattern")]
    #[test_case(&["--type", "j"], "invalid value 'j' for '--type" ; "unknown type")]
    #[test_case(&["--size", "one"], "invalid value 'one' for '--size" ; "non-numeric size")]
    #[test_case(&["--mtime", "13p"], "invalid value '13p' for '--mtime" ; "invalid mtime duration")]
    fn invalid(args: &[&str], expected: &str) -> Result<()> {
        Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .args(args)
            .assert()
            .stderr(predicate::str::contains(expected))
            .failure();

        Ok(())
    }

    fn setup_root_dir() -> Result<tempfile::TempDir> {
        let temp = tempfile::TempDir::new()?;
        let root = temp.path();

        // root/one/two
        fs::create_dir_all(root.join("one").join("two"))?;
        // root/three
        fs::create_dir(root.join("three"))?;

        // root/a.txt
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o744)
            .open(root.join("a.txt"))?;

        // root/one/b.txt
        let mut b_txt = fs::File::create(root.join("one").join("b.md"))?;
        write!(b_txt, "# Header")?;

        // root/one/two/c.txt
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o444)
            .open(root.join("one").join("two").join("c.txt"))?;

        // root/three/d.txt
        std::os::unix::fs::symlink(
            root.join("one").join("a.txt"),
            root.join("three").join("d.txt"),
        )?;

        Ok(temp)
    }
}
