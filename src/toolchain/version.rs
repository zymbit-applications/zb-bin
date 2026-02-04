use anyhow::{Context, Result};
use octocrab::models::repos::Release;
use urlencoding::encode;

/// `tag_prefix`: "zbcli" in "zbcli-1.1.0"
pub async fn list(
    tag_prefix: &str,
    n_items: u8,
) -> Result<Vec<Release>> {
    let github_instance = octocrab::instance();
    let repo = github_instance.repos("zymbit-applications", "zb-bin");

    let releases = repo.releases();

    let release_list = {
        let mut count = 0usize;
        let mut page = 1u32;
        let mut accumulate = Vec::new();
        while count < (n_items as usize) {
            accumulate.extend(
                releases
                    .list()
                    .per_page(n_items)
                    .page(page)
                    .send()
                    .await
                    .map(IntoIterator::into_iter)
                    .or_else(|e|
                        // don't bail if we get no results back unless we haven't accumulated any
                        // from previous pages
                        if accumulate.is_empty() {
                            Err(e)
                        } else {
                            Ok(Vec::new().into_iter())
                        })
                    .context("Failed to get latest releases")?
                    .filter(|release| release.tag_name.starts_with(tag_prefix)
                                && !release.tag_name.contains("rc")), // exclude release candidates
            );
            if count == accumulate.len() {
                break;
            }
            count = accumulate.len();
            page += 1;
        }
        accumulate.truncate(n_items as usize);
        accumulate
    };

    Ok(release_list)
}

pub async fn get_release(version: &str) -> Result<Release> {
    let github_instance = octocrab::instance();
    let repo = github_instance.repos("zymbit-applications", "zb-bin");
    let releases = repo.releases();

    let release = if version.to_lowercase().eq("latest") {
        releases
            .get_latest()
            .await
            .context("Failed to get latest release")?
    } else {
        let version_tag = encode(version).into_owned();
        releases
            .get_by_tag(version_tag.as_str())
            .await
            .context(format!("Failed to get release tagged with '{version}'.\n"))?
    };

    Ok(release)
}
