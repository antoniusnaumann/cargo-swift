# cargo-swift
[![Crates.io Version](https://img.shields.io/crates/v/cargo-swift)](https://crates.io/crates/cargo-swift)
[![CI](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/ci.yml/badge.svg)](https://github.com/antoniusnaumann/cargo-swift/actions)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-F46623)](https://www.rust-lang.org)

> A cargo plugin to easily build Swift packages from Rust code

It provides interactive commands for initializing and packaging a Rust library as Swift Package for usage in iOS and macOS apps.

This plugin heavily builds on [swift-bridge](https://github.com/chinedufn/swift-bridge), so if you like the bridging magic between Swift and Rust, check out swift-bridge!

## Getting Started
> **Note**
> This plugin can only be used on macOS, since proprietary toolchains are 
> required for this plugin to work properly.

### Prerequisites
Install this plugin
```
cargo install cargo-swift
```
Make sure the toolchain for your desired platforms are installed. (In the future, cargo-swift will install the required toolchains on demand if they are not present). To install toolchains for all macOS and iOS targets, run:
```
rustup target add x86_64-apple-darwin aarch64-apple-darwin aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
```
> **Note**
> Only Swift Packages that target Apple platforms can include binaries at the moment. 
> 
> If you need to target Linux or Windows, you might want to manually setup [swift-bridge](https://github.com/chinedufn/swift-bridge) instead.

### Using cargo-swift
You can create a new library template by running
```
cargo swift init
```
This creates a new Rust library crate with some boilerplate code and some examples to quickly get started with swift-bridge. For full reference, check out [this chapter of the Swift Bridge Book](https://chinedufn.github.io/swift-bridge/bridge-module/index.html)

To bundle the previously created Rust library as Swift Package, run:
```
cargo swift package
```
This command interactively prompts you for target platforms and swift package name.

That's it! You can now include the created package in an iOS or macOS app via Swift Package Manager.

