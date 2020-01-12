tinydnsproxy changelog
======================

## 0.2.0

The aim of this release is to replace crates that have a native component with crates written in pure Rust. This it to make cross-compiling for static ARM targets easier.

* Replaced native_tls with rustls

## 0.1.0

Initial release. Has the following features:

* Proxies plaintext DNS requests to a user-defined set of DNS-over-TLS providers
* Filters requests according to user-defined blocklists
* Has support for automatically reloading the blocklists over HTTP(S)
