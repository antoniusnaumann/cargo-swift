use crate::bindings::platform_headers_dir;
use crate::lib_type::LibType;
use crate::targets::ApplePlatform;
use crate::{Mode, Result, Target};
use anyhow::{anyhow, Context};
use std::fs::{self, remove_dir_all};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

    // Copy header files and modulemap from generated/<platform>/headers/
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

#[allow(clippy::too_many_arguments)]
pub fn create_xcframework(
    targets: &[Target],
    lib_name: &str,
    xcframework_name: &str,
    generated_dir: &Path,
    output_dir: &Path,
    mode: Mode,
    lib_type: LibType,
    privacy_manifest: Option<&Path>,
    bundle_identifier: Option<&str>,
) -> Result<()> {
    let output_dir_name = &output_dir
        .to_str()
        .ok_or(anyhow!("Output directory has an invalid name!"))?;

    let framework = format!("{output_dir_name}/{xcframework_name}.xcframework");

    let mut xcodebuild = Command::new("xcodebuild");
    xcodebuild.arg("-create-xcframework");

    match lib_type {
        LibType::Static => {
            // xcodebuild copies the headers directory as-is into each slice's `Headers/`,
            // keeping the per-module `Headers/<ffi_module_name>/` subdirectories.
            for target in targets {
                xcodebuild.arg("-library");
                xcodebuild.arg(target.library_path(lib_name, mode, lib_type));
                xcodebuild.arg("-headers");
                xcodebuild.arg(platform_headers_dir(generated_dir, target.platform()));
            }
        }
        LibType::Dynamic => {
            let default_id = format!("com.cargo-swift.{xcframework_name}");
            let bundle_id = bundle_identifier.unwrap_or(&default_id);

            for target in targets {
                let dylib_path = target.library_path(lib_name, mode, lib_type);
                let lib_dir = PathBuf::from(target.library_directory(mode));

                let fw_path = create_framework_bundle(
                    &dylib_path,
                    xcframework_name,
                    bundle_id,
                    &platform_headers_dir(generated_dir, target.platform()),
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
            }
        }
    }

    let output = xcodebuild
        .arg("-output")
        .arg(&framework)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        Err(output.stderr.into())
    } else {
        Ok(())
    }
}
