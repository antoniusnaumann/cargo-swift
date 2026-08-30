use crate::console::Error;
use crate::lib_type::LibType;
use crate::metadata::{metadata, MetadataExt};
use crate::targets::{library_file_name, ApplePlatform};
use crate::{Mode, Result, Target};
use anyhow::{anyhow, Context};
use std::fs::{self, remove_dir_all, DirEntry};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn search_subframework_paths(output_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut xcf_path: Option<DirEntry> = None;
    for sub_dir in std::fs::read_dir(output_dir)?.flatten() {
        if sub_dir
            .file_name()
            .to_str()
            .ok_or(anyhow!(
                "The directory that is being checked if it is an XCFramework has an invalid name!"
            ))?
            .contains(".xcframework")
        {
            xcf_path = Some(sub_dir)
        }
    }
    let mut subframework_paths = Vec::<PathBuf>::new();
    if let Some(path) = xcf_path {
        for subdir in std::fs::read_dir(path.path())? {
            let subdir = subdir?;
            let subdir_path = subdir.path();
            if subdir.file_type()?.is_dir() {
                subframework_paths.push(subdir_path);
            }
        }
    } else {
        return Err(Error::new(format!(
            "failed to find .xcframework in {output_dir:?}"
        )));
    }
    Ok(subframework_paths)
}

pub fn patch_subframework(
    sf_dir: &Path,
    generated_dir: &Path,
    ffi_module_name: &str,
) -> Result<()> {
    // xcodebuild creates lowercase "headers", but we rename to uppercase "Headers" (Apple convention)
    let mut headers = sf_dir.to_owned();
    headers.push("headers");
    remove_dir_all(&headers)
        .with_context(|| format!("Failed to remove unpatched directory {headers:?}"))?;
    let mut generated_headers = generated_dir.to_owned();
    generated_headers.push("headers");

    let mut patched_headers = sf_dir.to_owned();
    patched_headers.push("Headers");
    patched_headers.push(ffi_module_name);
    std::fs::create_dir_all(&patched_headers)
        .with_context(|| format!("Failed to create empty patched directory {patched_headers:?}"))?;

    let mut gen_header_files = Vec::<PathBuf>::new();
    for file in std::fs::read_dir(&generated_headers).with_context(|| {
        format!("Failed to read from the generated header directory {patched_headers:?}")
    })? {
        let file = file?;
        gen_header_files.push(file.path());
    }

    for path in gen_header_files {
        let filename = path
            .components()
            .next_back()
            .ok_or(anyhow!("Expected source filename when copying"))?;
        patched_headers.push(filename);
        std::fs::copy(&path, &patched_headers).with_context(|| {
            format!("Failed to copy header file from {path:?} to {patched_headers:?}")
        })?;
        let _copied_file = patched_headers.pop();
    }

    Ok(())
}

pub fn patch_xcframework(
    output_dir: &Path,
    generated_dir: &Path,
    ffi_module_name: &str,
) -> Result<()> {
    let subframeworks =
        search_subframework_paths(output_dir).context("Failed to get subframework components")?;
    for subframework in subframeworks {
        patch_subframework(&subframework, generated_dir, ffi_module_name)
            .with_context(|| format!("Failed to patch {subframework:?}"))?;
    }

    Ok(())
}

/// Creates a .framework bundle wrapping a dynamic library for a single platform slice.
///
/// iOS/tvOS/watchOS/visionOS use the flat ("shallow") layout:
/// ```text
/// {framework_name}.framework/
/// ├── Info.plist
/// ├── {framework_name}     (the dylib, renamed)
/// ├── Headers/
/// └── Modules/
/// ```
///
/// macOS and Mac Catalyst require the historical "versioned" layout, where the
/// real contents live under `Versions/A/` and the bundle root contains symlinks
/// pointing into `Versions/Current/`:
/// ```text
/// {framework_name}.framework/
/// ├── {framework_name}     -> Versions/Current/{framework_name}
/// ├── Headers              -> Versions/Current/Headers
/// ├── Modules              -> Versions/Current/Modules
/// ├── Resources            -> Versions/Current/Resources
/// └── Versions/
///     ├── A/
///     │   ├── {framework_name}
///     │   ├── Headers/
///     │   ├── Modules/
///     │   └── Resources/Info.plist
///     └── Current          -> A
/// ```
fn create_framework_bundle(
    dylib_path: &str,
    framework_name: &str,
    bundle_identifier: &str,
    headers_dir: &Path,
    output_dir: &Path,
    platform: ApplePlatform,
    privacy_manifest: Option<&Path>,
) -> Result<PathBuf> {
    let framework_dir = output_dir.join(format!("{framework_name}.framework"));

    // Clean up any previous framework bundle
    if framework_dir.exists() {
        remove_dir_all(&framework_dir)
            .with_context(|| format!("Failed to remove old framework bundle {framework_dir:?}"))?;
    }

    // Pick the directory where the actual binary/headers/modulemap/Info.plist live.
    // For shallow bundles this is the framework root; for versioned bundles it's
    // Versions/A and Info.plist goes into Versions/A/Resources.
    let versioned = platform.uses_versioned_bundle();
    let content_root = if versioned {
        framework_dir.join("Versions").join("A")
    } else {
        framework_dir.clone()
    };
    let info_plist_dir = if versioned {
        content_root.join("Resources")
    } else {
        content_root.clone()
    };

    let headers_dst = content_root.join("Headers");
    let modules_dst = content_root.join("Modules");
    fs::create_dir_all(&headers_dst)
        .with_context(|| format!("Failed to create Headers dir in {content_root:?}"))?;
    fs::create_dir_all(&modules_dst)
        .with_context(|| format!("Failed to create Modules dir in {content_root:?}"))?;
    fs::create_dir_all(&info_plist_dir)
        .with_context(|| format!("Failed to create Info.plist dir {info_plist_dir:?}"))?;

    // Copy dylib → {framework_name} (strip lib prefix and .dylib extension)
    let binary_dst = content_root.join(framework_name);
    fs::copy(dylib_path, &binary_dst).with_context(|| {
        format!("Failed to copy dylib from {dylib_path} to {binary_dst:?}")
    })?;

    // Run install_name_tool to set the framework rpath
    let install_name = Command::new("install_name_tool")
        .arg("-id")
        .arg(format!("@rpath/{framework_name}.framework/{framework_name}"))
        .arg(&binary_dst)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run install_name_tool")?;

    if !install_name.status.success() {
        return Err(anyhow!(
            "install_name_tool failed: {}",
            String::from_utf8_lossy(&install_name.stderr)
        )
        .into());
    }

    // Copy header files and modulemap from generated/headers/
    for entry in fs::read_dir(headers_dir)
        .with_context(|| format!("Failed to read headers dir {headers_dir:?}"))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name() else {
            continue;
        };

        if path.extension().is_some_and(|ext| ext == "modulemap") {
            // Patch "module X" → "framework module X" for framework bundles
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read modulemap from {path:?}"))?;
            let patched = content.replace("module ", "framework module ");
            fs::write(modules_dst.join(name), patched).with_context(|| {
                format!("Failed to write patched modulemap from {path:?}")
            })?;
        } else {
            fs::copy(&path, headers_dst.join(name)).with_context(|| {
                format!("Failed to copy header from {path:?}")
            })?;
        }
    }

    // Write Info.plist
    let plist = platform.info_plist();
    let min_version =
        std::env::var(plist.version_env_var).unwrap_or_else(|_| plist.default_version.to_owned());
    let device_family_block = if plist.device_family.is_empty() {
        String::new()
    } else {
        let items = plist
            .device_family
            .iter()
            .map(|d| format!("        <integer>{d}</integer>"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("    <key>UIDeviceFamily</key>\n    <array>\n{items}\n    </array>\n")
    };
    let supported_platform = plist.supported_platform;
    let version_key = plist.version_key;
    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>{framework_name}</string>
    <key>CFBundleIdentifier</key>
    <string>{bundle_identifier}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>{framework_name}</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>{supported_platform}</string>
    </array>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>{version_key}</key>
    <string>{min_version}</string>
{device_family_block}</dict>
</plist>
"#
    );
    fs::write(info_plist_dir.join("Info.plist"), info_plist)
        .context("Failed to write framework Info.plist")?;

    if let Some(manifest) = privacy_manifest {
        let dst = info_plist_dir.join("PrivacyInfo.xcprivacy");
        fs::copy(manifest, &dst).with_context(|| {
            format!("Failed to copy privacy manifest from {manifest:?} to {dst:?}")
        })?;
    }

    if versioned {
        create_versioned_symlinks(&framework_dir, framework_name)
            .context("Failed to create framework symlinks")?;
    }

    Ok(framework_dir)
}

/// For versioned (macOS / Mac Catalyst) frameworks, create the standard symlinks
/// pointing from the bundle root into `Versions/Current/`.
fn create_versioned_symlinks(framework_dir: &Path, framework_name: &str) -> Result<()> {
    // Versions/Current -> A
    symlink("A", framework_dir.join("Versions").join("Current"))
        .context("Failed to create Versions/Current symlink")?;

    for top_level in ["Headers", "Modules", "Resources"] {
        symlink(
            format!("Versions/Current/{top_level}"),
            framework_dir.join(top_level),
        )
        .with_context(|| format!("Failed to create {top_level} symlink"))?;
    }

    symlink(
        format!("Versions/Current/{framework_name}"),
        framework_dir.join(framework_name),
    )
    .context("Failed to create top-level binary symlink")?;

    Ok(())
}

/// Per-arch dSYM location produced by Cargo when `[profile.<mode>]` has
/// `debug = "limited"` (or higher) + `split-debuginfo = "packed"` on macOS:
/// Cargo invokes dsymutil during the link, dropping a bundle next to the dylib.
fn arch_dsym_path(arch: &str, lib_name: &str, mode: Mode) -> PathBuf {
    let target_dir = metadata().target_dir();
    let mode_dir = match mode {
        Mode::Debug => "debug",
        Mode::Release => "release",
    };
    let dsym_name = format!("{}.dSYM", library_file_name(lib_name, LibType::Dynamic));
    PathBuf::from(format!("{target_dir}/{arch}/{mode_dir}/deps/{dsym_name}"))
}

/// Materialise a dSYM bundle that matches a Target's framework binary:
/// - Single-arch targets: copy the per-arch dSYM with its DWARF Mach-O renamed
///   to `<framework_name>`.
/// - Universal targets: `lipo -create` the per-arch DWARF Mach-Os into a fat
///   binary at the renamed location. The result's embedded UUIDs cover every
///   arch in the corresponding framework binary, so Xcode auto-loads symbols
///   regardless of which slice the consumer is running.
///
/// Returns the staged dSYM path, ready to pass to
/// `xcodebuild -create-xcframework -debug-symbols`. The staged bundle is named
/// `<framework_name>.framework.dSYM` with the inner DWARF binary renamed to
/// `<framework_name>` so xcodebuild can match it to the framework slice and
/// drop it into `<slice>/dSYMs/` with the canonical name Xcode/App Store
/// Connect expects.
fn prepare_target_dsym(
    target: &Target,
    lib_name: &str,
    framework_name: &str,
    mode: Mode,
    staging_dir: &Path,
) -> Result<PathBuf> {
    let arches = target.architectures();
    let per_arch_dsyms: Vec<PathBuf> = arches
        .iter()
        .map(|arch| arch_dsym_path(arch, lib_name, mode))
        .collect();

    let missing: Vec<&PathBuf> = per_arch_dsyms.iter().filter(|p| !p.exists()).collect();
    if !missing.is_empty() {
        return Err(anyhow!(
            "--debug-symbols was passed but no .dSYM was found for target '{}'.\n\
             Expected dSYMs at:\n  {}\n\
             Enable debug info on the Cargo profile, e.g. add to [profile.{}]:\n\
                 debug = \"limited\"\n\
                 split-debuginfo = \"packed\"",
            target.display_name(),
            missing
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  "),
            match mode {
                Mode::Debug => "dev",
                Mode::Release => "release",
            },
        )
        .into());
    }

    let src_dwarf_name = library_file_name(lib_name, LibType::Dynamic);
    // Per-target subdir avoids collisions when multiple targets stage a
    // `<framework_name>.framework.dSYM` simultaneously.
    let target_key = match target {
        Target::Universal { universal_name, .. } => (*universal_name).to_string(),
        Target::Single { architecture, .. } => (*architecture).to_string(),
    };
    let staged = staging_dir
        .join(target_key)
        .join(format!("{framework_name}.framework.dSYM"));
    if staged.exists() {
        remove_dir_all(&staged)
            .with_context(|| format!("Failed to clean stale staged dSYM {staged:?}"))?;
    }

    // Copy the first per-arch dSYM as a template (gives us the bundle wrapper,
    // Info.plist, and resource layout). We then rename the DWARF Mach-O and
    // (for universal targets) overwrite it with the lipo'd fat binary.
    copy_dir_recursively(&per_arch_dsyms[0], &staged)
        .with_context(|| format!("Failed to stage dSYM from {:?}", per_arch_dsyms[0]))?;

    let dwarf_dir = staged.join("Contents/Resources/DWARF");
    let staged_dwarf_src = dwarf_dir.join(&src_dwarf_name);
    let staged_dwarf_dst = dwarf_dir.join(framework_name);
    if staged_dwarf_src.exists() {
        fs::rename(&staged_dwarf_src, &staged_dwarf_dst).with_context(|| {
            format!(
                "Failed to rename DWARF Mach-O from {staged_dwarf_src:?} to {staged_dwarf_dst:?}"
            )
        })?;
    }

    // Rewrite CFBundleIdentifier so the dSYM Info.plist reports the framework
    // identifier (com.apple.xcode.dsym.<framework>.framework) instead of the
    // value cargo's dsymutil inherited from the raw .dylib build artifact
    // (com.apple.xcode.dsym.lib<crate>.dylib). Symbolication keys on Mach-O
    // UUIDs, so this is cosmetic — but it's what Apple's tooling emits for
    // archive-built frameworks and what crash-reporter vendors expect.
    let info_plist = staged.join("Contents/Info.plist");
    if info_plist.exists() {
        let bundle_id = format!("com.apple.xcode.dsym.{framework_name}.framework");
        let status = Command::new("plutil")
            .args(["-replace", "CFBundleIdentifier", "-string"])
            .arg(&bundle_id)
            .arg(&info_plist)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .context("Failed to run plutil on dSYM Info.plist")?;
        if !status.success() {
            return Err(anyhow!(
                "plutil failed rewriting CFBundleIdentifier in {info_plist:?}"
            )
            .into());
        }
    }

    if per_arch_dsyms.len() > 1 {
        let mut lipo = Command::new("lipo");
        for d in &per_arch_dsyms {
            lipo.arg(d.join("Contents/Resources/DWARF").join(&src_dwarf_name));
        }
        lipo.arg("-create").arg("-output").arg(&staged_dwarf_dst);

        let lipo_out = lipo
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to invoke lipo on dSYM DWARF binaries")?;
        if !lipo_out.status.success() {
            return Err(anyhow!(
                "lipo failed combining dSYMs for target '{}': {}",
                target.display_name(),
                String::from_utf8_lossy(&lipo_out.stderr)
            )
            .into());
        }
    }

    Ok(staged)
}

/// Minimal recursive copy. dSYMs are plain directory trees of files and
/// (occasionally) symlinks; no special-casing needed for Resource forks etc.
fn copy_dir_recursively(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("Failed to create {dst:?}"))?;
    for entry in fs::read_dir(src).with_context(|| format!("Failed to read {src:?}"))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_entry = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursively(&entry.path(), &dst_entry)?;
        } else if file_type.is_symlink() {
            let link_target = fs::read_link(entry.path())?;
            symlink(link_target, &dst_entry).with_context(|| {
                format!("Failed to recreate symlink at {dst_entry:?}")
            })?;
        } else {
            fs::copy(entry.path(), &dst_entry).with_context(|| {
                format!("Failed to copy {:?} → {dst_entry:?}", entry.path())
            })?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn create_xcframework(
    targets: &[Target],
    lib_name: &str,
    xcframework_name: &str,
    ffi_module_name: &str,
    generated_dir: &Path,
    output_dir: &Path,
    mode: Mode,
    lib_type: LibType,
    privacy_manifest: Option<&Path>,
    bundle_identifier: Option<&str>,
    debug_symbols: bool,
) -> Result<()> {
    let output_dir_name = &output_dir
        .to_str()
        .ok_or(anyhow!("Output directory has an invalid name!"))?;

    let framework = format!("{output_dir_name}/{xcframework_name}.xcframework");

    // Stage dSYMs under cargo's target dir so they're cheaply cleanup-able and
    // don't pollute the package output. xcodebuild copies them into the
    // xcframework, so the staging dir is only needed during this invocation.
    let dsym_staging: Option<PathBuf> = if debug_symbols {
        let target_dir = metadata().target_dir();
        let dir = PathBuf::from(format!("{target_dir}/.cargo-swift-dsyms/{xcframework_name}"));
        if dir.exists() {
            remove_dir_all(&dir)
                .with_context(|| format!("Failed to clean stale dSYM staging dir {dir:?}"))?;
        }
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create dSYM staging dir {dir:?}"))?;
        Some(dir)
    } else {
        None
    };

    let mut xcodebuild = Command::new("xcodebuild");
    xcodebuild.arg("-create-xcframework");

    match lib_type {
        LibType::Static => {
            let libs: Vec<_> = targets
                .iter()
                .map(|t| t.library_path(lib_name, mode, lib_type))
                .collect();

            let headers = generated_dir.join("headers");
            let headers = headers
                .to_str()
                .ok_or(anyhow!("Directory for bindings has an invalid name!"))?;

            for (target, lib) in targets.iter().zip(libs.iter()) {
                xcodebuild.arg("-library");
                xcodebuild.arg(lib);
                xcodebuild.arg("-headers");
                xcodebuild.arg(headers);
                if let Some(staging) = &dsym_staging {
                    let dsym =
                        prepare_target_dsym(target, lib_name, xcframework_name, mode, staging)?;
                    // xcodebuild rejects relative paths for -debug-symbols.
                    let dsym = dsym
                        .canonicalize()
                        .with_context(|| format!("Failed to canonicalize dSYM {dsym:?}"))?;
                    xcodebuild.arg("-debug-symbols");
                    xcodebuild.arg(&dsym);
                }
            }
        }
        LibType::Dynamic => {
            let headers_dir = generated_dir.join("headers");
            let default_id = format!("com.cargo-swift.{xcframework_name}");
            let bundle_id = bundle_identifier.unwrap_or(&default_id);

            for target in targets {
                let dylib_path = target.library_path(lib_name, mode, lib_type);
                let lib_dir = PathBuf::from(target.library_directory(mode));

                let fw_path = create_framework_bundle(
                    &dylib_path,
                    xcframework_name,
                    bundle_id,
                    &headers_dir,
                    &lib_dir,
                    target.platform(),
                    privacy_manifest,
                )
                .with_context(|| {
                    format!(
                        "Failed to create framework bundle for target {}",
                        target.display_name()
                    )
                })?;

                xcodebuild.arg("-framework");
                xcodebuild.arg(&fw_path);
                if let Some(staging) = &dsym_staging {
                    let dsym =
                        prepare_target_dsym(target, lib_name, xcframework_name, mode, staging)?;
                    // xcodebuild rejects relative paths for -debug-symbols.
                    let dsym = dsym
                        .canonicalize()
                        .with_context(|| format!("Failed to canonicalize dSYM {dsym:?}"))?;
                    xcodebuild.arg("-debug-symbols");
                    xcodebuild.arg(&dsym);
                }
            }
        }
    }

    let output = xcodebuild
        .arg("-output")
        .arg(&framework)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    // Best-effort cleanup of the staging dir regardless of xcodebuild's outcome.
    if let Some(staging) = &dsym_staging {
        let _ = remove_dir_all(staging);
    }

    if !output.status.success() {
        Err(output.stderr.into())
    } else {
        // Only patch headers for static libraries — for dynamic, headers are already
        // inside each .framework bundle and xcodebuild preserves them as-is.
        if matches!(lib_type, LibType::Static) {
            patch_xcframework(output_dir, generated_dir, ffi_module_name)
                .context("Failed to patch the XCFramework")?;
        }
        Ok(())
    }
}
