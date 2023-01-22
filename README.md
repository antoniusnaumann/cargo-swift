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

## License
### Apache-2.0
```
 Copyright 2023 Antonius Naumann

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```
### MIT
```
MIT License

Copyright (c) 2023 Antonius Naumann

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
