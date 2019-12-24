#!/bin/bash -e

function terminate() {
    msg="$1"
    code="$2"
    [[ -z "$code" ]] && code="0"
    printf -- "Terminating: %s\n" "$msg"
    exit
}

# Check for required programs
[[ -z "$(command -v openssl)" ]] && terminate "Cannot find 'openssl'" "1"

# Create list of block lists
declare -A lists
lists[EasyList]="https://example.com"
