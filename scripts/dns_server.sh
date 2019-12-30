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

grepcmd=""
if [[ "$(uname)" = "Darwin" ]]; then
    grepcmd="ggrep"

    [[ -z "$(command -v ${grepcmd})" ]] && terminate "Cannot find GNU grep (brew install grep)" "1"
else
    grepcmd="grep"
fi

if [[ "${1}" = "--help" ]] || [[ -z "${1}" ]]; then
    cat <<EOF

Usage: dns_server.sh ip_address [port (optional - defaults to 853)]

Example: dns_server.sh 8.8.8.8 853

This script helps create a '[[dns_server]]' section for tinydnsproxy's config.toml files. You just need to provide it with an IP address (and optionally a port) and it will output a TOML section you can just paste into your config.

EOF
    exit 0
fi


# Validate arguments to the script
server="$1"
port="853"
[[ -z "${2}" ]] || port="$2"

# Get the server common name
cn="$(openssl s_client -showcerts -connect ${server}:853 </dev/null 2>/dev/null | openssl x509 -subject | ${grepcmd} -Po 'CN=\K.*')"

[[ "${?}" != "0" ]] && terminate "openssl couldn't connect to supplied IP" "1"

# Construct the final string

output=$(cat <<EOF
[[dns_server]]
ip_address = "${server}"
port = ${port}
hostname = "${cn}"
EOF
)

printf -- "\n%s\n\n" "${output}"
