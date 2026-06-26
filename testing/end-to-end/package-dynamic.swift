#!/usr/bin/env swift
import Foundation

func error(_ msg: String) { FileHandle.standardError.write(msg.data(using: .utf8)!) }
func dirExists(atPath path: String) -> Bool {
    var isDirectory : ObjCBool = true
    let exists = FileManager.default.fileExists(atPath: path, isDirectory: &isDirectory)
    return exists && isDirectory.boolValue
}
func fileExists(atPath path: String) -> Bool {
    var isDirectory : ObjCBool = true
    let exists = FileManager.default.fileExists(atPath: path, isDirectory: &isDirectory)
    return exists && !isDirectory.boolValue
}

let projectName = "swift-project-dynamic"
let libName = "swift_project_dynamic"
let packageName = "SwiftProjectDynamic"

// Create project
print("Creating project...")
let cargoSwiftInit = Process()
cargoSwiftInit.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftInit.arguments = ["cargo", "swift", "init", projectName, "-y", "--silent"]
try! cargoSwiftInit.run()
cargoSwiftInit.waitUntilExit()

guard cargoSwiftInit.terminationStatus == 0 else {
    error("cargo swift init failed with status \(cargoSwiftInit.terminationStatus)")
    exit(1)
}

// Patch Cargo.toml to use cdylib
print("Patching Cargo.toml for cdylib...")
let cargoTomlPath = "\(projectName)/Cargo.toml"
var cargoToml = try! String(contentsOfFile: cargoTomlPath, encoding: .utf8)
cargoToml = cargoToml.replacingOccurrences(
    of: "crate-type = [\"staticlib\", \"lib\"]",
    with: "crate-type = [\"cdylib\", \"lib\"]"
)
// Make this a multi-crate (multi-component) build: add a second UniFFI crate as a path
// dependency + workspace member. This exercises the framework-module-per-component path.
cargoToml = cargoToml.replacingOccurrences(
    of: "[dependencies]\n",
    with: "[dependencies]\ndep_crate = { path = \"dep_crate\" }\n"
)
cargoToml += "\n[workspace]\nmembers = [\"dep_crate\"]\n"
try! cargoToml.write(toFile: cargoTomlPath, atomically: true, encoding: .utf8)

// Second component: its own crate with setup_scaffolding!() (a separate UniFFI namespace).
print("Adding second crate (dep_crate)...")
try! FileManager.default.createDirectory(
    atPath: "\(projectName)/dep_crate/src", withIntermediateDirectories: true
)
try! """
[package]
name = "dep_crate"
version = "0.1.0"
edition = "2021"

[dependencies]
uniffi = "0.31"
""".write(toFile: "\(projectName)/dep_crate/Cargo.toml", atomically: true, encoding: .utf8)
try! """
uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn dep_hello() -> String { "dep".to_string() }
""".write(toFile: "\(projectName)/dep_crate/src/lib.rs", atomically: true, encoding: .utf8)

// Reference dep_crate from the main crate so its component is linked into the cdylib.
let mainLibPath = "\(projectName)/src/lib.rs"
var mainLib = try! String(contentsOfFile: mainLibPath, encoding: .utf8)
mainLib += "\n#[uniffi::export]\npub fn dep_passthrough() -> String { dep_crate::dep_hello() }\n"
try! mainLib.write(toFile: mainLibPath, atomically: true, encoding: .utf8)

// Add uniffi.toml with custom ffi_module_name
print("Adding uniffi.toml with custom ffi_module_name...")
let ffiModuleName = "CustomDynFFI"
let uniffiToml = """
[bindings.swift]
ffi_module_name = "\(ffiModuleName)"
"""
FileManager.default.createFile(
    atPath: "\(projectName)/uniffi.toml",
    contents: uniffiToml.data(using: .utf8),
    attributes: nil
)

// Drop a PrivacyInfo.xcprivacy alongside the project so we can pass it via
// --privacy-manifest and assert it gets embedded everywhere.
print("Adding PrivacyInfo.xcprivacy...")
let privacyManifest = """
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
\t<key>NSPrivacyTracking</key>
\t<false/>
\t<key>NSPrivacyTrackingDomains</key>
\t<array/>
\t<key>NSPrivacyCollectedDataTypes</key>
\t<array/>
\t<key>NSPrivacyAccessedAPITypes</key>
\t<array/>
</dict>
</plist>
"""
let privacyManifestPath = "\(projectName)/PrivacyInfo.xcprivacy"
FileManager.default.createFile(
    atPath: privacyManifestPath,
    contents: privacyManifest.data(using: .utf8),
    attributes: nil
)

// Package as dynamic library
print("Running cargo swift package --lib-type dynamic...")
let cargoSwiftPackage = Process()
let xcFrameworkName = ffiModuleName
cargoSwiftPackage.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftPackage.currentDirectoryPath += "/" + projectName
let bundleIdentifier = "io.test.dynamic.\(ffiModuleName)"
cargoSwiftPackage.arguments = [
    "cargo", "swift", "package", "-y", "--silent",
    "-p", "macos", "ios",
    "--lib-type", "dynamic",
    "--privacy-manifest", "PrivacyInfo.xcprivacy",
    "--bundle-identifier", bundleIdentifier,
]
try! cargoSwiftPackage.run()
cargoSwiftPackage.waitUntilExit()

guard cargoSwiftPackage.terminationStatus == 0 else {
    error("cargo swift package --lib-type dynamic failed with status \(cargoSwiftPackage.terminationStatus)")
    exit(1)
}

// Verify basic package structure
guard dirExists(atPath: "\(projectName)/\(packageName)") else {
    error("No package directory (\"\(packageName)/\") found in project directory")
    exit(1)
}
guard fileExists(atPath: "\(projectName)/\(packageName)/Package.swift") else {
    error("No Package.swift file found in package directory")
    exit(1)
}
guard dirExists(atPath: "\(projectName)/\(packageName)/\(xcFrameworkName).xcframework") else {
    error("No .xcframework directory found in package directory (expected \(xcFrameworkName).xcframework)")
    exit(1)
}
guard dirExists(atPath: "\(projectName)/\(packageName)/Sources") else {
    error("No \"Sources/\" directory found in package directory")
    exit(1)
}
guard fileExists(atPath: "\(projectName)/\(packageName)/Sources/\(packageName)/\(libName).swift") else {
    error("No \(libName).swift file found in module")
    exit(1)
}

// --privacy-manifest should drop a copy at the Swift package root so SPM
// auto-bundles it alongside Package.swift.
guard fileExists(atPath: "\(projectName)/\(packageName)/PrivacyInfo.xcprivacy") else {
    error("No PrivacyInfo.xcprivacy at Swift package root (expected via --privacy-manifest)")
    exit(1)
}

// Verify that xcframework contains .framework bundles (not bare dylibs)
let xcframeworkPath = "\(projectName)/\(packageName)/\(xcFrameworkName).xcframework"
let subframeworks = try! FileManager.default.contentsOfDirectory(atPath: xcframeworkPath)
    .filter { !$0.hasPrefix(".") && $0 != "Info.plist" }

guard !subframeworks.isEmpty else {
    error("XCFramework has no platform slices")
    exit(1)
}

// macOS and Mac Catalyst use the versioned bundle layout (Versions/A/... with
// top-level symlinks). All other Apple platforms use the shallow layout.
func usesVersionedBundle(slice: String) -> Bool {
    return slice.hasPrefix("macos") || slice.contains("maccatalyst")
}

func isSymlink(atPath path: String) -> Bool {
    let attrs = try? FileManager.default.attributesOfItem(atPath: path)
    return (attrs?[.type] as? FileAttributeType) == .typeSymbolicLink
}

for subframework in subframeworks {
    let slicePath = "\(xcframeworkPath)/\(subframework)"

    // Each slice should contain a .framework bundle
    let frameworkPath = "\(slicePath)/\(xcFrameworkName).framework"
    guard dirExists(atPath: frameworkPath) else {
        error("Expected .framework bundle at \(frameworkPath) — got bare dylib instead?")
        exit(1)
    }

    let versioned = usesVersionedBundle(slice: subframework)

    // Where the actual content lives in the bundle.
    let contentRoot = versioned ? "\(frameworkPath)/Versions/A" : frameworkPath
    let infoPlistPath = versioned
        ? "\(contentRoot)/Resources/Info.plist"
        : "\(contentRoot)/Info.plist"

    // Binary
    guard fileExists(atPath: "\(contentRoot)/\(xcFrameworkName)") else {
        error("No binary found at \(contentRoot)/\(xcFrameworkName)")
        exit(1)
    }

    // Info.plist
    guard fileExists(atPath: infoPlistPath) else {
        error("No Info.plist found at \(infoPlistPath)")
        exit(1)
    }

    // Headers
    guard dirExists(atPath: "\(contentRoot)/Headers") else {
        error("No Headers/ directory found in \(contentRoot)")
        exit(1)
    }
    guard fileExists(atPath: "\(contentRoot)/Headers/\(xcFrameworkName).h") else {
        error("No \(xcFrameworkName).h found in \(contentRoot)/Headers/")
        exit(1)
    }

    // Modules/module.modulemap
    guard dirExists(atPath: "\(contentRoot)/Modules") else {
        error("No Modules/ directory found in \(contentRoot)")
        exit(1)
    }
    guard fileExists(atPath: "\(contentRoot)/Modules/module.modulemap") else {
        error("No module.modulemap found in \(contentRoot)/Modules/")
        exit(1)
    }

    // --privacy-manifest should embed PrivacyInfo.xcprivacy alongside Info.plist
    // (framework root for shallow, Versions/A/Resources for versioned).
    let privacyInfoPath = versioned
        ? "\(contentRoot)/Resources/PrivacyInfo.xcprivacy"
        : "\(contentRoot)/PrivacyInfo.xcprivacy"
    guard fileExists(atPath: privacyInfoPath) else {
        error("No PrivacyInfo.xcprivacy embedded at \(privacyInfoPath)")
        exit(1)
    }

    // Info.plist must carry the platform-version + supported-platforms keys or
    // App Store Connect rejects the upload (error 90530 / 90360).
    let plist = NSDictionary(contentsOfFile: infoPlistPath) as? [String: Any] ?? [:]
    let versionKey = subframework.hasPrefix("macos") || subframework.contains("maccatalyst")
        ? "LSMinimumSystemVersion"
        : "MinimumOSVersion"
    guard plist[versionKey] is String else {
        error("\(slicePath): Info.plist missing \(versionKey)")
        exit(1)
    }
    guard let supported = plist["CFBundleSupportedPlatforms"] as? [String], !supported.isEmpty else {
        error("\(slicePath): Info.plist missing CFBundleSupportedPlatforms")
        exit(1)
    }

    // --bundle-identifier should be used as CFBundleIdentifier
    guard let actualId = plist["CFBundleIdentifier"] as? String, actualId == bundleIdentifier else {
        let actualId = plist["CFBundleIdentifier"] as? String ?? "<missing>"
        error("\(slicePath): CFBundleIdentifier should be \(bundleIdentifier), got \(actualId)")
        exit(1)
    }

    // Versioned-bundle-only checks: required symlinks at the bundle root and
    // Versions/Current pointing at A.
    if versioned {
        guard isSymlink(atPath: "\(frameworkPath)/Versions/Current") else {
            error("\(slicePath): Versions/Current is not a symlink")
            exit(1)
        }
        for top in ["Headers", "Modules", "Resources", xcFrameworkName] {
            guard isSymlink(atPath: "\(frameworkPath)/\(top)") else {
                error("\(slicePath): expected symlink at \(frameworkPath)/\(top)")
                exit(1)
            }
        }
    } else {
        // Shallow bundles must NOT have a Versions/ directory.
        if dirExists(atPath: "\(frameworkPath)/Versions") {
            error("\(slicePath): shallow bundle should not contain Versions/")
            exit(1)
        }
    }

    print("  \(subframework): \(versioned ? "versioned" : "shallow") .framework structure verified")
}

// Verify install_name_tool set the correct rpath
print("Checking install name on framework binaries...")
for subframework in subframeworks {
    let binaryPath = "\(xcframeworkPath)/\(subframework)/\(xcFrameworkName).framework/\(xcFrameworkName)"
    let otool = Process()
    let pipe = Pipe()
    otool.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    otool.arguments = ["otool", "-D", binaryPath]
    otool.standardOutput = pipe
    try! otool.run()
    otool.waitUntilExit()

    let output = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
    let expectedId = "@rpath/\(xcFrameworkName).framework/\(xcFrameworkName)"
    guard output.contains(expectedId) else {
        error("Binary install name should contain \(expectedId), got:\n\(output)")
        exit(1)
    }
    print("  \(subframework): install name OK")
}

// --- Multi-crate regression checks (#97/#99/#100) ---
// The bundle must expose exactly one framework module (named after the bundle), not one per
// component, and every generated Swift file must import that single module.
print("Checking merged framework module...")
for subframework in subframeworks {
    let frameworkPath = "\(xcframeworkPath)/\(subframework)/\(xcFrameworkName).framework"
    let shallow = "\(frameworkPath)/Modules/module.modulemap"
    let versioned = "\(frameworkPath)/Versions/A/Modules/module.modulemap"
    let mmPath = fileExists(atPath: shallow) ? shallow : versioned
    let mm = (try? String(contentsOfFile: mmPath, encoding: .utf8)) ?? ""
    let moduleCount = mm.components(separatedBy: "framework module ").count - 1
    guard moduleCount == 1 else {
        error("\(subframework): expected exactly 1 framework module, found \(moduleCount)\n\(mm)")
        exit(1)
    }
}

// There must be >=2 component Swift files (multi-crate), all importing the single module.
let sourcesDir = "\(projectName)/\(packageName)/Sources/\(packageName)"
let swiftFiles = try! FileManager.default.contentsOfDirectory(atPath: sourcesDir)
    .filter { $0.hasSuffix(".swift") }
guard swiftFiles.count >= 2 else {
    error("Expected >=2 component Swift files (multi-crate), found \(swiftFiles)")
    exit(1)
}

// Type-check the generated sources the way Xcode resolves framework modules: by bundle name,
// via -F. `swift build` loads the modulemap directly and does not surface this, so -F
// resolution is what guards it. (#97/#99)
print("Type-checking generated sources with -F (Xcode-style framework resolution)...")
let macosSlice = subframeworks.first { $0.hasPrefix("macos") }!
let swiftc = Process()
swiftc.executableURL = URL(fileURLWithPath: "/usr/bin/env")
swiftc.arguments = ["swiftc", "-typecheck", "-module-name", packageName,
                    "-F", "\(xcframeworkPath)/\(macosSlice)"]
    + swiftFiles.map { "\(sourcesDir)/\($0)" }
try! swiftc.run()
swiftc.waitUntilExit()
guard swiftc.terminationStatus == 0 else {
    error("swiftc -F failed: generated sources do not resolve under Xcode-style framework module resolution")
    exit(1)
}
print("  -F resolution OK")

// Build the Swift package to verify it links correctly
print("Building Swift package...")
let swift = Process()
swift.executableURL = URL(fileURLWithPath: "/usr/bin/env")
swift.currentDirectoryPath += "/\(projectName)/\(packageName)"
swift.arguments = ["swift", "build"]
try! swift.run()
swift.waitUntilExit()

guard swift.terminationStatus == 0 else {
    error("Swift build failed")
    exit(1)
}

print("Tests for cargo swift package --lib-type dynamic passed!")
