# FHC
FHC stands for Fast HTTP Checker, it's written in Rust. Works on Linux, Windows, macOS, Android, Aarch64, ARM and possibly in your oven.

# Goal
Offer the community an efficient HTTP checker tool.

# Methodology
FHC try to resolve https first, if this fails then fallback to http, in that way you don't miss any active HTTP host.

# Performance & speed
FHC is **very** resource friendly, you can use up to 1000 threads in an single core machine and this will work without any problem, the bottleneck for this tool is your network speed. By default, FHC is able to perform HTTP check for ~913 hosts per second in good network conditions (tested in an Google Cloud machine). Depending on how much host have only http (not https) and/or are alive the number of host resolved in average can be low/higher as the tool have or not to perform a double check. In our demo we used an real-world scenario performing resolution for `google.com` subdomains.

## Demo
The hosts file used in the demo is [here](files/hosts.txt).

[![asciicast](https://asciinema.org/a/363640.svg)](https://asciinema.org/a/363640)

# Installation

## Using precompiled binaries.

Download the asset from the [releases page](https://github.com/Edu4rdSHL/fhc/releases/latest) according to your platform.

## Using the source code.

1. You need to have the lastest stable [Rust](https://www.rust-lang.org/) version insalled in your system.
2. Clone the repo or download the source code, then run `cargo build --release`.
3. Execute the tool from `./target/release/fhc` or add it to your system PATH to use from anywhere.

# Usage
* Show all HTTP urls depite their response codes:
```
cat hosts.txt | fhc
```
* If you want to see only the HTTP host with 200-299 codes:
```
cat hosts.txt | fhc -2
```
You can tune the `--timeout`, `-t/--threads`, `-u/--user-agent` and other options according to your needs. See `fhc --help`
