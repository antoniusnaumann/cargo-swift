# Configuration
> **Warning**
> This section describes a feature that is not yet (fully) implemented.
> If you have thoughts on this draft, feel free to comment on the relevant issues in the issue section or [open a new one](https://github.com/antoniusnaumann/cargo-swift/issues)!

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

