use crate::console::Error;
use crate::lib_type::LibType;
use crate::{Mode, Result, Target};
use anyhow::{anyhow, Context};
use std::fs::{remove_dir_all, DirEntry};
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
    xcframework_name: &str,
) -> Result<()> {
    let mut headers = sf_dir.to_owned();
    headers.push("headers");
    remove_dir_all(&headers)
        .with_context(|| format!("Failed to remove unpatched directory {headers:?}"))?;
    let mut generated_headers = generated_dir.to_owned();
    generated_headers.push("headers");

    let mut patched_headers = sf_dir.to_owned();
    patched_headers.push("headers");
    patched_headers.push(xcframework_name);
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
    xcframework_name: &str,
) -> Result<()> {
    let subframeworks =
        search_subframework_paths(output_dir).context("Failed to get subframework components")?;
    for subframework in subframeworks {
        patch_subframework(&subframework, generated_dir, xcframework_name)
            .with_context(|| format!("Failed to patch {subframework:?}"))?;
    }

    Ok(())
}
pub fn create_xcframework(
    targets: &[Target],
    lib_name: &str,
    xcframework_name: &str,
    generated_dir: &Path,
    output_dir: &Path,
    mode: Mode,
    lib_type: LibType,
) -> Result<()> {
    /*println!(
        "Targets: {:#?}\nlib_name: {:?}\nxcframework_name: {:?}\ngenerated_dir {:?}\noutput_dir: {:?}\nmode: {:?}\nlib_type: {:?}",
        targets, lib_name, xcframework_name, generated_dir, output_dir, mode, lib_type
    );*/
    let libs: Vec<_> = targets
        .iter()
        .map(|t| t.library_path(lib_name, mode, lib_type))
        .collect();

    let headers = generated_dir.join("headers");
    let headers = headers
        .to_str()
        .ok_or(anyhow!("Directory for bindings has an invalid name!"))?;

    let output_dir_name = &output_dir
        .to_str()
        .ok_or(anyhow!("Output directory has an invalid name!"))?;

    let framework = format!("{output_dir_name}/{xcframework_name}.xcframework");

    let mut xcodebuild = Command::new("xcodebuild");
    xcodebuild.arg("-create-xcframework");

    for lib in &libs {
        xcodebuild.arg("-library");
        xcodebuild.arg(lib);
        xcodebuild.arg("-headers");
        xcodebuild.arg(headers);
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
        patch_xcframework(output_dir, generated_dir, xcframework_name)
            .context("Failed to patch the XCFramework")?;
        Ok(())
    }
}
