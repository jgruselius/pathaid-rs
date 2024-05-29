#! /bin/bash

# Joel Gruselius 2024

list_path() {
  while read -d : -r x; do
    echo -e "$x"
  done < <(echo "$PATH")
}

ls_path() {
  while read -d : -r x; do
    files=()
    IFS=$'\n' read -r -d '' -A files < <(find -L "$x" -maxdepth 1 -type f -executable)
    echo -e "$x: ${#files[@]}"
  done < <(echo "$PATH")
}

function append_path {
  case ":$PATH:" in
    *:"$1":*)
      ;;
    *)
      echo -n "${PATH:+$PATH:}${1}"
  esac
}

function prepend_path {
  case ":$PATH:" in
    *:"$1":*)
      ;;
    *)
      echo -n "${1}:${PATH:+$PATH}"
  esac
}

function dedup_path {
  perl -e 'print join(":", grep { not $seen{$_}++ } split(/:/, $ENV{PATH}))'
}
