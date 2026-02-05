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

print("Creating project...")
let cargoSwiftInit = Process()
let projectName = "swift-project"
let libName = "swift_project"
let packageName = "SwiftProject"
cargoSwiftInit.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftInit.arguments = ["cargo", "swift", "init", projectName, "-y", "--silent"]

try! cargoSwiftInit.run()
cargoSwiftInit.waitUntilExit()

// Test: uniffi.toml ffi_module_name should be respected
// The xcframework name is now derived from the FFI module name (--xcframework-name is deprecated)
print("Adding uniffi.toml with custom ffi_module_name...")
let ffiModuleName = "CustomFFI"
let uniffiToml = """
[bindings.swift]
ffi_module_name = "\(ffiModuleName)"
"""
FileManager.default.createFile(atPath: "\(projectName)/uniffi.toml", contents: uniffiToml.data(using: .utf8), attributes: nil)

print("Running tests for cargo swift package...")
let cargoSwiftPackage = Process()
// Note: --xcframework-name is deprecated; xcframework name is now derived from ffi_module_name
let xcFrameworkName = ffiModuleName  // Should match ffi_module_name from uniffi.toml
cargoSwiftPackage.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftPackage.currentDirectoryPath += "/" + projectName
cargoSwiftPackage.arguments = ["cargo", "swift", "package", "-y", "--silent", "-p", "macos", "ios"]

try! cargoSwiftPackage.run()
cargoSwiftPackage.waitUntilExit()

guard cargoSwiftPackage.terminationStatus == 0 else {
    error("cargo swift package failed with status \(cargoSwiftPackage.terminationStatus)")
    exit(1)
}

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
guard dirExists(atPath: "\(projectName)/\(packageName)/Sources/\(packageName)") else {
    error("No \"\(packageName)/\" directory found in sources directory")
    exit(1)
}
guard fileExists(atPath: "\(projectName)/\(packageName)/Sources/\(packageName)/\(libName).swift") else {
    error("No \(libName).swift file found in module")
    exit(1)
}

// Verify that the FFI module name from uniffi.toml is used for the Headers folder
// Headers should be uppercase (Apple convention)
let xcframeworkPath = "\(projectName)/\(packageName)/\(xcFrameworkName).xcframework"
let subframeworks = try! FileManager.default.contentsOfDirectory(atPath: xcframeworkPath)
    .filter { !$0.hasPrefix(".") && $0 != "Info.plist" }

for subframework in subframeworks {
    let headersPath = "\(xcframeworkPath)/\(subframework)/Headers/\(ffiModuleName)"
    guard dirExists(atPath: headersPath) else {
        error("Headers folder should use ffi_module_name from uniffi.toml: expected \(headersPath)")
        exit(1)
    }

    let headerFile = "\(headersPath)/\(ffiModuleName).h"
    guard fileExists(atPath: headerFile) else {
        error("Header file should use ffi_module_name from uniffi.toml: expected \(headerFile)")
        exit(1)
    }
}

print("Verified: uniffi.toml ffi_module_name is respected")

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

print("Tests for cargo swift package passed!")
