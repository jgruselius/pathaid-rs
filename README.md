A simple CLI for the `pathops` module I needed for another project. You could use it to check what's in the `PATH` variable, remove any duplicates and add paths to it.

I wrote it so it can replace the functions I had defined in my dotfiles (`path_helper.sh`), but it has more fancy output!

```
Simple tool to validate the PATH environment variable

Usage: pathaid [COMMAND]

Commands:
  list      List entries (default)
  validate  Validate all entries
  dedup     Remove any duplicate entries
  count     Count executables
  append    Add a directory to end of PATH and print the result
  prepend   Add a directory to front of PATH and print the result
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
``` 
