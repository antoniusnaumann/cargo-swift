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
let packageName = "SwiftProject"
cargoSwiftInit.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftInit.arguments = ["cargo", "swift", "init", projectName, "-y", "--silent"]

try! cargoSwiftInit.run()
cargoSwiftInit.waitUntilExit()

print("Running tests for cargo swift package...")
let cargoSwiftPackage = Process()
cargoSwiftPackage.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftPackage.currentDirectoryPath += "/" + projectName
cargoSwiftPackage.arguments = ["cargo", "swift", "package", "-y", "--silent", "-p", "macos", "ios"]

try! cargoSwiftPackage.run()
cargoSwiftPackage.waitUntilExit()

guard dirExists(atPath: "\(projectName)/\(packageName)") else { 
	error("No package directory (\"\(packageName)/\") found in project directory")
	exit(1)
}
guard fileExists(atPath: "\(projectName)/\(packageName)/Package.swift") else { 
	error("No Package.swift file found in package directory")
	exit(1)
}
guard dirExists(atPath: "\(projectName)/\(packageName)/RustXcframework.xcframework") else { 
	error("No .xcframework directory found in package directory")
	exit(1)
}
guard dirExists(atPath: "\(projectName)/\(packageName)/Sources") else { 
	error("No \"Sources/\" directory found in package directory")
	exit(1)
}

print("Tests for cargo swift package passed!")
