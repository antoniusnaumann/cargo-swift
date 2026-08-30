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

/// Run a process to completion, capture stdout, return as String.
func capture(_ args: [String]) throws -> String {
    let p = Process()
    let pipe = Pipe()
    p.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    p.arguments = args
    p.standardOutput = pipe
    p.standardError = Pipe()
    try p.run()
    p.waitUntilExit()
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    return String(data: data, encoding: .utf8) ?? ""
}

/// Parse `dwarfdump --uuid` output for a Mach-O binary into the set of (UUID, arch)
/// pairs it contains. Each line looks like:
///     UUID: 6A8B... (arm64) /path/to/binary
func uuidsForBinary(at path: String) throws -> Set<String> {
    let out = try capture(["dwarfdump", "--uuid", path])
    var ids = Set<String>()
    for line in out.split(separator: "\n") {
        // grab "UUID: <hex> (<arch>)"
        guard let r = line.range(of: #"UUID:\s+([0-9A-Fa-f-]+)\s+\(([^)]+)\)"#, options: .regularExpression)
        else { continue }
        let snippet = String(line[r])
        let parts = snippet.replacingOccurrences(of: "UUID:", with: "").trimmingCharacters(in: .whitespaces)
        ids.insert(parts)
    }
    return ids
}

let projectName = "swift-project-debug-symbols"
let libName = "swift_project_debug_symbols"
let packageName = "SwiftProjectDebugSymbols"
let ffiModuleName = "DebugSymbolsFFI"
let xcFrameworkName = ffiModuleName

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

// Switch to cdylib + add the release profile knobs that produce .dSYM bundles
// (`debug` + `split-debuginfo = packed` runs dsymutil during link on macOS).
print("Patching Cargo.toml: cdylib + release profile with split-debuginfo...")
let cargoTomlPath = "\(projectName)/Cargo.toml"
var cargoToml = try! String(contentsOfFile: cargoTomlPath, encoding: .utf8)
cargoToml = cargoToml.replacingOccurrences(
    of: "crate-type = [\"staticlib\", \"lib\"]",
    with: "crate-type = [\"cdylib\", \"lib\"]"
)
cargoToml += """

[profile.release]
debug = "limited"
split-debuginfo = "packed"
"""
try! cargoToml.write(toFile: cargoTomlPath, atomically: true, encoding: .utf8)

// Add uniffi.toml with custom ffi_module_name so the framework binary name is
// deterministic (DebugSymbolsFFI) regardless of the crate name munging.
print("Adding uniffi.toml with custom ffi_module_name...")
let uniffiToml = """
[bindings.swift]
ffi_module_name = "\(ffiModuleName)"
"""
FileManager.default.createFile(
    atPath: "\(projectName)/uniffi.toml",
    contents: uniffiToml.data(using: .utf8),
    attributes: nil
)

// Package as dynamic library with --debug-symbols + --release so the dSYMs end
// up in dSYMs/ subdirs inside the xcframework.
print("Running cargo swift package --debug-symbols --release...")
let cargoSwiftPackage = Process()
cargoSwiftPackage.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftPackage.currentDirectoryPath += "/" + projectName
cargoSwiftPackage.arguments = [
    "cargo", "swift", "package", "-y", "--silent",
    "-p", "macos", "ios",
    "--lib-type", "dynamic",
    "--release",
    "--debug-symbols",
    "--bundle-identifier", "io.test.dsyms.\(ffiModuleName)",
]
try! cargoSwiftPackage.run()
cargoSwiftPackage.waitUntilExit()
guard cargoSwiftPackage.terminationStatus == 0 else {
    error("cargo swift package --debug-symbols failed with status \(cargoSwiftPackage.terminationStatus)")
    exit(1)
}

let xcframeworkPath = "\(projectName)/\(packageName)/\(xcFrameworkName).xcframework"
guard dirExists(atPath: xcframeworkPath) else {
    error("xcframework not produced at \(xcframeworkPath)")
    exit(1)
}

let subframeworks = try! FileManager.default.contentsOfDirectory(atPath: xcframeworkPath)
    .filter { !$0.hasPrefix(".") && $0 != "Info.plist" }

guard !subframeworks.isEmpty else {
    error("xcframework has no platform slices")
    exit(1)
}

// macOS and Mac Catalyst use the versioned bundle layout. All other Apple
// platforms use the shallow layout — the framework binary path differs.
func usesVersionedBundle(slice: String) -> Bool {
    return slice.hasPrefix("macos") || slice.contains("maccatalyst")
}

// For every slice: verify dSYMs/<framework>.framework.dSYM exists with the
// expected internal structure, and that its DWARF binary's UUID set matches
// the framework binary's UUID set so Xcode auto-loads symbols at runtime.
for subframework in subframeworks {
    let slicePath = "\(xcframeworkPath)/\(subframework)"
    let dsymRoot = "\(slicePath)/dSYMs"
    let dsymBundle = "\(dsymRoot)/\(xcFrameworkName).framework.dSYM"
    let dsymDwarf = "\(dsymBundle)/Contents/Resources/DWARF/\(xcFrameworkName)"
    let dsymInfoPlist = "\(dsymBundle)/Contents/Info.plist"

    // 1. dSYMs/ subdir is what xcodebuild -create-xcframework -debug-symbols
    // produces. If it's missing, the flag never made it through.
    guard dirExists(atPath: dsymRoot) else {
        error("\(slicePath): missing dSYMs/ subdir — was -debug-symbols passed?")
        exit(1)
    }

    // 2. The dSYM bundle and its inner Mach-O must both exist.
    guard dirExists(atPath: dsymBundle) else {
        error("\(slicePath): missing \(xcFrameworkName).framework.dSYM bundle")
        exit(1)
    }
    guard fileExists(atPath: dsymDwarf) else {
        error("\(slicePath): missing DWARF binary at \(dsymDwarf)")
        exit(1)
    }
    guard fileExists(atPath: dsymInfoPlist) else {
        error("\(slicePath): missing dSYM Info.plist at \(dsymInfoPlist)")
        exit(1)
    }

    // 2b. CFBundleIdentifier should match the framework's dSYM convention
    // (com.apple.xcode.dsym.<framework>.framework) — not the raw .dylib name
    // that cargo's dsymutil emits. Cosmetic but matches what Xcode produces
    // for archive-built frameworks and what crash-reporter vendors expect.
    let plist = NSDictionary(contentsOfFile: dsymInfoPlist) as? [String: Any] ?? [:]
    let expectedID = "com.apple.xcode.dsym.\(xcFrameworkName).framework"
    guard (plist["CFBundleIdentifier"] as? String) == expectedID else {
        error("\(slicePath): dSYM CFBundleIdentifier should be \(expectedID), got \(plist["CFBundleIdentifier"] ?? "<nil>")")
        exit(1)
    }
    guard (plist["CFBundlePackageType"] as? String) == "dSYM" else {
        error("\(slicePath): dSYM CFBundlePackageType should be 'dSYM'")
        exit(1)
    }

    // 3. UUID parity. dSYMs are matched to binaries by Mach-O LC_UUID; if the
    // dSYM's UUID set isn't a superset of the framework binary's, lldb /
    // App Store Connect can't symbolicate.
    let versioned = usesVersionedBundle(slice: subframework)
    let frameworkBinary = versioned
        ? "\(slicePath)/\(xcFrameworkName).framework/Versions/A/\(xcFrameworkName)"
        : "\(slicePath)/\(xcFrameworkName).framework/\(xcFrameworkName)"
    let binaryUUIDs = try uuidsForBinary(at: frameworkBinary)
    let dsymUUIDs = try uuidsForBinary(at: dsymDwarf)
    guard !binaryUUIDs.isEmpty else {
        error("\(slicePath): could not read UUIDs from framework binary \(frameworkBinary)")
        exit(1)
    }
    guard binaryUUIDs.isSubset(of: dsymUUIDs) else {
        let missing = binaryUUIDs.subtracting(dsymUUIDs).joined(separator: ", ")
        error("\(slicePath): dSYM is missing UUIDs that the framework binary has: \(missing)")
        exit(1)
    }

    print("  \(subframework): dSYM OK, UUIDs \(dsymUUIDs) cover binary \(binaryUUIDs)")
}

print("Tests for cargo swift package --debug-symbols passed!")
