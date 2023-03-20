# cargo-swift
[![Crates.io Version](https://img.shields.io/crates/v/cargo-swift)](https://crates.io/crates/cargo-swift)
[![E2E Tests](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/end-to-end.yml/badge.svg)](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/end-to-end.yml)
[![Build](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/ci.yml/badge.svg)](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/ci.yml)
[![Publish](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/publish.yml/badge.svg)](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/publish.yml)
[![Clippy](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/clippy.yml/badge.svg)](https://github.com/antoniusnaumann/cargo-swift/actions/workflows/clippy.yml)
[![Language: Rust](https://img.shields.io/badge/Language-Rust-F46623)](https://www.rust-lang.org)

> A cargo plugin to easily build Swift packages from Rust code

*cargo swift* provides interactive commands for initializing and packaging a Rust library as Swift Package for usage in iOS and macOS apps.
This plugin uses Mozilla's [UniFFI](https://github.com/mozilla/uniffi-rs) for bridging between Swift and Rust. To learn more about using UniFFI, read its [User Guide](https://mozilla.github.io/uniffi-rs/Overview.html),
but note that you can skip the parts about generating bindings and building a swift module as *cargo swift* already takes care of this!

![](https://github.com/antoniusnaumann/cargo-swift/blob/main/readme/cargo-swift-demo.gif)

## Getting Started
> **Note**
> This plugin can only be used on macOS, since proprietary toolchains are 
> required for this plugin to work properly.

### Prerequisites
Install this plugin, simply run
```
cargo install cargo-swift
```

### Using cargo-swift
You can create a new library crate by running
```
cargo swift init
```
This creates a new Rust library crate with some boilerplate code and some examples to quickly get started with UniFFI. For full reference, check out [this chapter of the UniFFI User Guide](https://mozilla.github.io/uniffi-rs/udl_file_spec.html)

To bundle the previously created Rust library as Swift Package, run:
```
cargo swift package
```
This command interactively prompts you for swift package name and target platforms.
If some required toolchains for the selected target platforms are missing, cargo swift will ask you if it should install them automatically.

That's it! You can now include the created package in an iOS or macOS app via Swift Package Manager.

### Configuration
> **Warning**
> This section describes a feature that is not yet implemented.

Configuration options can be supplied to cargo-swift in multiple ways:

1. Command-line arguments (i.e. ```--platforms macos ios```)
2. Meta-data under ```[package.metadata.swiftpackage]``` tag in crate-level Cargo.toml
3. Meta-data under ```[workspace.metadata.swiftpackage]``` tag in workspace-level Cargo.toml
4. Prompt configuration values that are not provided by the methods above

These configuration values take precedence over each other in the order listed above, so an explicitly given command-line argument will always override a value given in the config file.

#### Metadata
```cargo swift package``` can be invoked with the ```--save``` option to store all given configuration values in the crate's Cargo.toml. Alternatively, they can be filled in manually.
The following configuration values can be included:

```TOML
# ...
[package.metadata.swiftpackage]
# Name of your package as seen by Swift (upper camel case is recommended)
name = "YourSwiftPackageName"
# Target platform identifiers (case-insensitive). Currently supported platforms are: macos, ios
platforms = ["ios", "macos"]
# ...
````

Target platforms may also be set in workspace-level Cargo.toml under a ```[workspace.metadata.swiftpackage]``` instead.

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
