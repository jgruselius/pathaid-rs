/*
Plan of path tool

# Commmands:

list           list all paths in PATH
validate       check for duplicate entries, non-existing or empty directories
dedup          remove any duplicates and print result
append         add one or more (separated by ':') paths to the end and print result
prepend        add one or more (separated by ':') paths to the front and print result
*/

mod pathops;

use std::collections::HashSet;
use anyhow::{Context, Result};
use clap::{arg, Command};
use colored::{ColoredString, Colorize};
use std::env;
use std::path::{Path, PathBuf};

fn fmt_path(path: impl AsRef<Path>, level: usize) -> ColoredString {
    let p = path.as_ref().to_string_lossy();
    match level {
        0 => p.blue(),
        1 => p.yellow(),
        2 => p.red(),
        _ => p.bold(),
    }
}

fn fmt_num(num: usize, level: usize) -> ColoredString {
    let n = format!("{}", num);
    match level {
        0 => n.magenta(),
        1 => n.yellow(),
        2 => n.red(),
        _ => n.bold(),
    }
}
fn list_paths() -> Result<()> {
    let path = pathops::get_path()?;
    let paths = pathops::split(path);
    for p in paths.iter() {
        // Print using different format for normal paths, those that refer to some other path,
        // and non-existing paths:
        if let Ok(res) = p.canonicalize() {
            if res.as_os_str() == p.as_os_str() {
                println!("{}", fmt_path(p, 0));
            } else {
                println!("{} -> {}", fmt_path(p, 1), fmt_path(res, 0));
            }
        } else {
            println!("{}", fmt_path(p, 2));
        }
    }

    Ok(())
}

fn validate() -> Result<()> {
    let path = pathops::get_path()?;
    let paths = pathops::split(path);
    for p in paths.iter() {
        if !pathops::exists(p) {
            println!("{} is not an accessible directory", fmt_path(p, 2));
        } else if pathops::is_empty(p)? {
            println!("{} is empty", fmt_path(p, 1));
        }
    }
    let dups = pathops::find_duplicates(&paths);
    if !dups.is_empty() {
        let unique_dups: HashSet<PathBuf> = dups.iter().map(|p| p.clone()).collect();
        for p in unique_dups.iter() {
            let n = dups.iter().filter(|&x| x == p).count();
            println!("{} is included {} times", fmt_path(p, 1), n + 1);
        }
    }
    /* Filter duplicate resolved paths to those that are different when resolved:
    let resolved_dups: Vec<PathBuf> = pathops::find_duplicates_resolved(&paths).into_iter()
        .filter(|p| !dups.contains(p))
        .collect();
    */

    let resolved_dups = pathops::find_duplicates_resolved(&paths);
    if !resolved_dups.is_empty() {
        let unique_dups: HashSet<PathBuf> = resolved_dups.iter().map(|p| p.clone()).collect();
        for p in unique_dups.iter() {
            let n = resolved_dups.iter().filter(|&x| x == p).count();
            println!("{} is included {} times when entries are resolved", fmt_path(p, 1), n + 1);
        }
    }

    Ok(())
}

fn dedup() -> Result<()> {
    let path = pathops::get_path()?;
    let paths = pathops::split(path);
    let resolved_dups = pathops::find_duplicates_resolved(&paths);
    if !resolved_dups.is_empty() {
        let info = format!("({} resolved duplicate entries removed)\n", resolved_dups.len());
        eprintln!("{}", info.dimmed());
    }
    let unique = pathops::dedup(&paths);
    let new_path = pathops::join(&unique)?;
    println!("{}", new_path);

    Ok(())
}

fn count_exes() -> Result<()> {
    let path = pathops::get_path()?;
    let paths = pathops::split(path);
    for p in paths.iter() {
        match pathops::count_files(p) {
            Ok(0) => println!("{}: {}", fmt_path(p, 1), 0),
            Ok(n) => println!("{}: {}", fmt_path(p, 0), n),
            _ => println!("{}: --", fmt_path(p, 2)),
        }
    }

    Ok(())
}

fn append_path(addition: impl AsRef<str>) -> Result<()> {
    let path = pathops::get_path()?;
    let addition = addition.as_ref();
    pathops::validate_addition(&path, addition)?;
    let new_path = pathops::append_path(&path, addition)?;
    println!("{}", new_path);

    Ok(())
}

fn prepend_path(addition: impl AsRef<str>) -> Result<()> {
    let path = pathops::get_path()?;
    let addition = addition.as_ref();
    pathops::validate_addition(&path, addition)?;
    let new_path = pathops::prepend_path(&path, addition)?;
    println!("{}", new_path);

    Ok(())
}

fn main() -> Result<()> {
    let parser = Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(Command::new("list").about("List entries"))
        .subcommand(Command::new("validate").about("Validate all entries"))
        .subcommand(Command::new("dedup").about("Remove any duplicate entries"))
        .subcommand(Command::new("count").about("Count executables"))
        .subcommand(
            Command::new("append")
                .about("Append directory")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Stuff to add")),
        )
        .subcommand(
            Command::new("prepend")
                .about("Prepend directory")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Stuff to add")),
        );

    let matches = parser.get_matches();
    match matches.subcommand() {
        Some(("validate", _)) => validate()?,
        Some(("dedup", _)) => dedup()?,
        Some(("count", _)) => count_exes()?,
        Some(("append", subm)) => {
            let p = subm.get_one::<String>("PATH").unwrap();
            append_path(p)?;
        }
        Some(("prepend", subm)) => {
            let p = subm.get_one::<String>("PATH").unwrap();
            prepend_path(p)?;
        }
        _ => list_paths()?,
    }

    Ok(())
}
