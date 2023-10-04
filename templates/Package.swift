// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.
// Swift Package: <PACKAGE_NAME>

import PackageDescription;

let package = Package(
    name: "<PACKAGE_NAME>",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_10)
    ],
    products: [
        .library(
            name: "<PACKAGE_NAME>",
            targets: ["<PACKAGE_NAME>"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "RustFramework", path: "./RustFramework.xcframework"),
        .target(
            name: "<PACKAGE_NAME>",
            dependencies: [
                .target(name: "RustFramework")
            ]
        ),
    ]
)
