use std::{fs::File, path::Path};

use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use octocrab::models::repos::Release;

use crate::{
    system,
    terminal::{formatted_left_output, OutputColor},
    toolchain::version,
    zbcli::ZbcliAsset,
};

pub async fn prompt_version(
    tag_prefix: &str,
    target_asset: ZbcliAsset,
) -> Result<String> {
    let releases = version::list(tag_prefix, 10_u8).await?;

    let asset_name = target_asset.to_string();
    let releases_list = releases
        .into_iter()
        .filter(|release| {
            release
                .assets
                .iter()
                .any(|asset| asset_name == asset.name)
        })
        .collect::<Vec<_>>();
    if releases_list.is_empty() {
        bail!("no matching releases found")
    }

    let releases_strings = releases_list
        .iter()
        .map(|release| &release.tag_name)
        .collect::<Vec<_>>();

    let selection = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select version")
        .items(&releases_strings)
        .interact()
        .context("Failed to get version selection")?;

    let mut releases_list = releases_list;
    Ok(releases_list.remove(selection).tag_name)
}

pub async fn install(target_release: Release, asset_type: ZbcliAsset) -> Result<()> {
    let asset_name = asset_type.to_string();
    let Some(asset) = target_release
        .assets
        .iter()
        .find(|asset| asset_name == asset.name)
    else {
        bail!(
            "{} failed to find '{}' in release '{}'",
            formatted_left_output("Error", &OutputColor::Red),
            asset_name,
            target_release.tag_name,
        )
    };

    let asset_res = reqwest::get(asset.browser_download_url.clone()).await?;
    let mut content = std::io::Cursor::new(asset_res.bytes().await?);

    println!("Installing zbcli");

    let zb_path = Path::new("/usr/bin/zbcli");

    let mut zb_file = File::create(zb_path)?;
    std::io::copy(&mut content, &mut zb_file)?;
    system::add_executable_permission(zb_path)?;

    Ok(())
}
