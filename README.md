tinydnsproxy
============

tinydnsproxy is a small program that acts as a local DNS resolver that relays requests on to DNS-over-TLS servers exclusively. I wrote this because:

* I wanted a small resolver for my local network;
* I wanted a resolver that forces all upstream requests to be over DNS-over-TLS;
* I wanted a resolver that uses and auto-updates blocklists;
* I wanted a small, focused, tool with only the features I need;
* I wanted to learn Rust on a project that sits in that lovely space between trivial and non-trivial.

## Caveats

Caveats first, because if you're considering this you'll almost certainly be better off with a proper resolver like Unbound or dnsmasq.

* tinydnsproxy only does normal TLS validation where the received certificate is checked against the operating system's trust store. Maybe in future I'll add optional certificate pinning;
* It hasn't been written with efficiency in mind, so you probably only want to deploy it on a local LAN;
* This doesn't *really* parse DNS messages. It does the bare minimum to extract hostnames and we do some byte patching to turn a request into a NXDOMAIN response;
* It's my (probably rubbish and unidiomatic) Rust.

## Usage

Everything is configured in a config file. Have a look at the [example config file](./config/example.toml) for exhaustive options. If you're not entirely confident creating a `[[dns_server]]` entry then there is [a script to create it for you](./scripts/dns_server.sh).

## Potential Improvements

* Introducing certificate pinning
* Adding a pool of TLS connections (currently every DNS query results in a fresh TLS connection being established)
* Add proper DNS message parsing and construction (looking for a nice library to do this)

## Things I'm Probably Not Going to Do

* Make it async :: threads are good enough for the time being;
* Add support for DNS-over-HTTPS :: not a huge amount of reason to when DNS-over-TLS exists;
* Add support for other protocols on the LAN side.
