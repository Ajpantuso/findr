// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod findr {
    use anyhow::Result;
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use tempfile;
    use test_case::test_case;

    #[test_case(&[r"--pattern=\.md$"], &["./one/b.md"] ; ".md files")]
    #[test_case(&["--type=d"], &[".", "./one", "./one/two", "./three"] ; "directories")]
    #[test_case(&["--type=x"], &["./a.txt"] ; "executables")]
    #[test_case(&["--type=l"], &["./three/d.txt"] ; "symlink")]
    #[test_case(&["--mode=444"], &["./one/two/c.txt"] ; "readonly")]
    #[test_case(&["--size=0"], &["./a.txt", "./one/two/c.txt"] ; "size equals 0")]
    #[test_case(&["--type=f", "--type=l", "--size=+0"], &["./one/b.md", "./three/d.txt"] ; "files/symlinks with size greater than 0")]
    #[test_case(&["--size=8"], &["./one/b.md"] ; "size equals 8")]
    #[test_case(&["--max-depth=1"], &[".", "./a.txt", "./one", "./three"] ; "max-depth equals 0")]
    #[test_case(&["--min-depth=3"], &["./one/two/c.txt"] ; "min-depth equals 3")]
    fn valid(args: &[&str], expected: &[&str]) -> Result<()> {
        let dir = setup_root_dir()?;

        Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .current_dir(dir.path())
            .args(args)
            .assert()
            .stdout(predicate::function(|out: &str| {
                let stdout_lines = out.lines().collect::<Vec<&str>>();
                let stdout_line_counts = count_lines(&stdout_lines);
                let expected_line_counts = count_lines(expected);

                stdout_line_counts == expected_line_counts
            }))
            .success();

        Ok(dir.close()?)
    }

    fn count_lines<'a>(lines: &'a [&str]) -> HashMap<&'a str, usize> {
        let mut counts: HashMap<&str, usize> = HashMap::new();

        for l in lines {
            counts.entry(l).and_modify(|n| *n += 1).or_default();
        }

        counts
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
