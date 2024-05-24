use askama::Template;

#[derive(Template)]
#[template(path = "template.toml", escape = "none")]
pub(crate) struct CargoToml<'a> {
    pub(crate) crate_name: &'a str,
    pub(crate) namespace: &'a str,
    // TODO: Use LibType::identifier here
    pub(crate) lib_type: &'a str,
    pub(crate) macro_only: bool,
}

#[derive(Template)]
#[template(path = "build.rs", escape = "none")]
pub(crate) struct BuildRs {}

#[derive(Template)]
#[template(path = "lib.rs", escape = "none")]
pub(crate) struct LibRs {
    pub(crate) plain: bool,
    pub(crate) macro_only: bool,
}

#[derive(Template)]
#[template(path = "lib.udl", escape = "none")]
pub(crate) struct LibUdl<'a> {
    pub(crate) namespace: &'a str,
    pub(crate) plain: bool,
}

#[derive(Template)]
#[template(path = "Package.swift", escape = "none")]
pub(crate) struct PackageSwift<'a> {
    pub(crate) package_name: &'a str,
    pub(crate) xcframework_name: &'a str,
    pub(crate) disable_warnings: bool,
}
