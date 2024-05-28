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

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Display, Path};
use anyhow::{Context, Result};
use colored::{ColoredString, Colorize};
use clap::{arg, Command};


fn fmt_path(path: &Path, level: usize) -> ColoredString {
    let p = path.to_string_lossy();
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
    let paths = pathops::split(&path);
    for p in paths.iter() {
        println!("{}", fmt_path(p, 0));
    }
    Ok(())
}

fn validate() -> Result<()> {
    let path = pathops::get_path()?;
    let paths = pathops::split(&path);
    let dups = pathops::find_duplicates(&paths);
    if !dups.is_empty() {
        println!("{} duplicate entries found:", fmt_num(dups.len(), 0));
        dups.iter().for_each(|p| println!("{}", fmt_path(p, 1)));
    }
    for p in paths.iter() {
        if !pathops::exists(p) {
            println!("{} is not an accessible directory", fmt_path(p, 2));
        } else if pathops::is_empty(p)? {
            println!("{} is empty", fmt_path(p, 1));
        }
    }

    Ok(())
}

fn count_exes() -> Result<()> {
    let path = pathops::get_path()?;
    let paths = pathops::split(&path);
    for p in paths.iter() {
        match pathops::count_files(p) {
            Ok(0) => println!("{}: {}", fmt_path(p, 1), 0),
            Ok(n) => println!("{}: {}", fmt_path(p, 0), n),
            _ => println!("{}: --", fmt_path(p, 2)),
        }
    }
    Ok(())
}

fn append_path(addition: &OsString) -> Result<()> {
    let path = pathops::get_path()?;

    pathops::validate_addition(&path, addition)?;
    let new_path = pathops::append_path(&path, addition)?;
    println!("{}", new_path.to_string_lossy());

    Ok(())
}

fn prepend_path(addition: &OsString) -> Result<()> {
    let path = pathops::get_path()?;

    pathops::validate_addition(&path, addition)?;
    let new_path = pathops::prepend_path(&path, addition)?;
    println!("{}", new_path.to_string_lossy());

    Ok(())
}

fn main() -> Result<()> {

    let parser = Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("list")
                .about("List entries")
        )
        .subcommand(
            Command::new("validate")
                .about("Validate all entries")
        )
        .subcommand(
            Command::new("count")
                .about("Count executables")
        )
        .subcommand(
            Command::new("append")
                .about("Append directory")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Stuff to add"))
        )
        .subcommand(
            Command::new("prepend")
                .about("Prepend directory")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Stuff to add"))
        );

    let matches = parser.get_matches();
    match matches.subcommand() {
        Some(("validate", _)) => validate()?,
        Some(("count", _)) => count_exes()?,
        Some(("append", subm)) => {
            let p: OsString = subm.get_one::<String>("PATH").unwrap().to_owned().into();
            append_path(&p)?;
        },
        Some(("prepend", subm)) => {
            let p: OsString = subm.get_one::<String>("PATH").unwrap().to_owned().into();
            prepend_path(&p)?;
        },
        _ => list_paths()?,
    }

    Ok(())
}
