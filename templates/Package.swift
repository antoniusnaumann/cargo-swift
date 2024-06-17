// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.
// Swift Package: {{ package_name }}

import PackageDescription;

let package = Package(
    name: "{{ package_name }}",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "{{ package_name }}",
            targets: ["{{ package_name }}"]
        )
    ],
    dependencies: [ ],
    targets: [
        .binaryTarget(name: "{{ xcframework_name }}", path: "./{{ xcframework_name }}.xcframework"),
        .target(
            name: "{{ package_name }}",
            dependencies: [
                .target(name: "{{ xcframework_name }}")
            ]{% if disable_warnings %},
            swiftSettings: [
                .unsafeFlags(["-suppress-warnings"]),
            ]
            {%- endif %}
        ),
    ]
)
