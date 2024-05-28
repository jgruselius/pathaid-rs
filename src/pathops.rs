/*
Joel Gruselius 2024

# Functions

# get the PATH environment variable
get_path() -> string

# split the string on ':'
split(string) -> list<string>

# check if path exists and is a directory
exists(string) -> bool

# check if path contains no executables (case of below)
is_empty(string) -> bool

# count all executables in a path
count(string) -> int

# check if there are duplicate entries
has_duplicates(string) -> bool

# list duplicate entries
get_duplicates(string) -> list<string>

# return all unique entries
dedup(string) -> list<string>

TODO:
    [ ] HashSet doesn't maintain order, re-implement
*/

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::collections::HashSet;
use anyhow::{Context, ensure, Result};

// Get the PATH environment variable
pub fn get_path() -> Result<OsString> {
    env::var_os("PATH").context("unable to fetch PATH environment variable")
}

// Split the string on ':' or ';' (Windows)
pub fn split(path_var: &OsString) -> Vec<PathBuf> {
    env::split_paths(path_var).collect()
}

pub fn join(paths: &Vec<PathBuf>) -> Result<OsString> {
    env::join_paths(paths).context("unable to join path components")
}

// Split and join via HashSet as internal functions for manipulating path:
fn split_hs(path_var: &OsString) -> HashSet<PathBuf> {
    env::split_paths(path_var).collect()
}

fn join_hs(paths: &HashSet<PathBuf>) -> Result<OsString> {
    env::join_paths(paths).context("unable to join path components")
}

// Check if path exists and is a directory
pub fn exists(path: &Path) -> bool {
    match path.canonicalize() {
        Ok(p) => p.exists() && p.is_dir(),
        _ => false,
    }
}

// Count all executables in a path
pub fn count_files(path: &Path) -> Result<usize> {
    Ok(fs::read_dir(path)?
        .filter_map(|d| d.ok().and_then(|p| p.path().canonicalize().ok()))
        .filter(|p| p.is_file())
        .count()
    )
}

// Check if path contains no executables (special case of count_files = 0)
pub fn is_empty(path: &Path) -> Result<bool> {
    Ok(count_files(path)? == 0)
}

// Get elements occurring more than once
pub fn find_duplicates(paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut duplicates: Vec<PathBuf> = Vec::new();

    for path in paths {
        if seen.contains(path) {
            duplicates.push(path.clone());
        } else {
            seen.insert(path.clone());
        }
    }
    duplicates
}

// Return unique entries while maintaining order
pub fn dedup(paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut unique: Vec<PathBuf> = Vec::new();

    for path in paths {
        if seen.contains(path) {
            unique.push(path.clone());
        } else {
            seen.insert(path.clone());
        }
    }
    unique
}

// Verify that addition is not already in path string
pub fn ensure_unique_addition(path_var: &OsString, addition: &OsString) -> Result<()> {
    let path_to_add = PathBuf::from(addition);
    let unique_paths = split_hs(path_var);
    ensure!(
        !unique_paths.contains(&path_to_add),
        format!("PATH already contains '{}'", addition.to_string_lossy())
    );
    Ok(())
}

// Add addition to the end of path_var
pub fn append_path(path_var: &OsString, addition: &OsString) -> Result<OsString> {
    // Now add while preserving order:
    let mut paths = split(path_var);
    paths.push(PathBuf::from(addition));
    join(&paths)
}

// Add addition to the front of path_var
pub fn prepend_path(path_var: &OsString, addition: &OsString) -> Result<OsString> {
    // Now add while preserving order:
    let mut paths = split(path_var);
    paths.insert(0, PathBuf::from(addition));
    join(&paths)
}

// Combine some tests
pub fn validate_addition(path_var: &OsString, addition: &OsString) -> Result<()> {
    let path_to_add = PathBuf::from(addition);
    ensure!(
        exists(&path_to_add),
        format!("'{}' is not an existing directory", addition.to_string_lossy())
    );
    ensure_unique_addition(path_var, addition)
}


#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Error};
    use super::*;

    struct Test {
        // An example path string
        path: OsString,
        // And its components
        paths: Vec<PathBuf>,
        // Entries occurring more than once in paths
        dups: Vec<PathBuf>,
        // Some unique path:
        addition: OsString,
        // An existing path
        exe_dir: PathBuf,
    }
    impl Test {
        fn new() -> Self {
            Self {
                path: OsString::from("/usr/local/bin:/usr/local/sbin:/usr/bin:/bin:/usr/local/bin"),
                paths: (&["/usr/local/bin", "/usr/local/sbin", "/usr/bin", "/bin", "/usr/local/bin"])
                    .into_iter()
                    .map(|s| PathBuf::from(s))
                    .collect(),
                addition: OsString::from("/unique/addition"),
                dups: vec![PathBuf::from("/usr/local/bin")],
                // Use the directory of the executing program:
                exe_dir: env::current_exe().unwrap().parent().unwrap().to_path_buf(),
            }
        }
    }

    #[test]
    fn test_get_path() {
        let p = get_path().unwrap();
        assert!(!p.is_empty());
    }

    #[test]
    fn test_split() {
        let test = Test::new();
        assert_eq!(split(&test.path), test.paths)
    }

    #[test]
    fn test_join() {
        let test = Test::new();
        let joined = join(&test.paths).unwrap();
        assert_eq!(joined, test.path)
    }

    #[test]
    fn test_exists() {
        let test = Test::new();
        assert!(exists(&test.exe_dir))
    }

    #[test]
    fn test_count_files() {
        let test = Test::new();
        let count = count_files(&test.exe_dir).unwrap();
        assert!(count > 0)
    }

    #[test]
    fn test_is_empty() {
        let test = Test::new();
        let res = is_empty(&test.exe_dir).unwrap();
        assert!(!res)
    }

    #[test]
    fn test_find_duplicates() {
        let test = Test::new();
        let dups = vec![PathBuf::from("/usr/local/bin")];
        assert_eq!(find_duplicates(&test.paths), dups)
    }

    #[test]
    fn test_dedup() {
        let test = Test::new();
        assert_eq!(find_duplicates(&test.paths), test.dups)
    }

    #[test]
    fn test_ensure_unique_addition() {
        let test = Test::new();
        ensure_unique_addition(&test.path, &test.addition).unwrap();
        let existing = test.paths.get(0).unwrap().clone().into_os_string();
        let res = std::panic::catch_unwind(
            || ensure_unique_addition(&test.path, &existing).unwrap()
        );
        assert!(res.is_err())
    }

    #[test]
    fn test_append_path() {
        let test = Test::new();
        let delim = if cfg!(windows) {
            ";"
        } else {
            ":"
        };
        let mut expected = test.path.clone();
        expected.push(format!("{}{}", delim, test.addition.to_string_lossy()));
        let res = append_path(&test.path, &test.addition).unwrap();
        assert_eq!(res, expected)
    }

    #[test]
    fn test_prepend_path() {
        let test = Test::new();
        let delim = if cfg!(windows) {
            ";"
        } else {
            ":"
        };
        let mut expected = OsString::from(format!("{}{}", test.addition.to_string_lossy(), delim));
        expected.push(&test.path);
        let res = prepend_path(&test.path, &test.addition).unwrap();
        assert_eq!(res, expected)
    }
}