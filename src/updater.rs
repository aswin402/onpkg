use anyhow::Result;

/// Check for updates and self-update the binary from GitHub Releases.
pub fn check_and_update(current_version: &str) -> Result<self_update::Status> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("aswin402")
        .repo_name("onpkg")
        .bin_name("onpkg")
        .show_download_progress(true)
        .current_version(current_version)
        .build()?
        .update()?;
    Ok(status)
}
