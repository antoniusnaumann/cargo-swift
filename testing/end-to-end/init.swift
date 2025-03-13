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

print("Running tests for cargo swift init in macro mode...")

let cargoSwiftInit = Process()
let projectName = "ExampleProject"
cargoSwiftInit.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftInit.arguments = ["cargo", "swift", "init", projectName, "-y", "--silent"]

try! cargoSwiftInit.run()
cargoSwiftInit.waitUntilExit()

guard dirExists(atPath: projectName) else {
	error("Project directory does not exist")
	exit(1) 
}
guard fileExists(atPath: "\(projectName)/Cargo.toml") else { 
	error("No Cargo.toml found in project directory")
	exit(1)
}
guard fileExists(atPath: "\(projectName)/.gitignore") else { 
	error("No .gitignore found in project directory")
	exit(1)
}
guard dirExists(atPath: "\(projectName)/src") else { 
	error("No src-directory found in project directory")
	exit(1)
}
guard fileExists(atPath: "\(projectName)/src/lib.rs") else { 
	error("No lib.rs file found in src directory")
	exit(1)
}

print("Running tests for cargo swift init in udl mode...")

let cargoSwiftInitUdl = Process()
let projectNameUdl = "\(projectName)_udl"
cargoSwiftInitUdl.executableURL = URL(fileURLWithPath: "/usr/bin/env")
cargoSwiftInitUdl.arguments = ["cargo", "swift", "init", projectNameUdl, "-y", "--silent", "--udl"]

try! cargoSwiftInitUdl.run()
cargoSwiftInitUdl.waitUntilExit()

guard dirExists(atPath: projectNameUdl) else {
	error("Project directory does not exist")
	exit(1) 
}
guard fileExists(atPath: "\(projectNameUdl)/Cargo.toml") else { 
	error("No Cargo.toml found in project directory")
	exit(1)
}
guard fileExists(atPath: "\(projectNameUdl)/build.rs") else { 
	error("No build.rs file found in project directory")
	exit(1)
}
guard fileExists(atPath: "\(projectNameUdl)/.gitignore") else { 
	error("No .gitignore found in project directory")
	exit(1)
}
guard dirExists(atPath: "\(projectNameUdl)/src") else { 
	error("No src-directory found in project directory")
	exit(1)
}
guard fileExists(atPath: "\(projectNameUdl)/src/lib.rs") else { 
	error("No lib.rs file found in src directory")
	exit(1)
}
guard fileExists(atPath: "\(projectNameUdl)/src/lib.udl") else { 
	error("No lib.udl file found in src directory")
	exit(1)
}

print("Tests for cargo swift init passed!")
