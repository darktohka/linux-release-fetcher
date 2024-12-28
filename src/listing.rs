pub mod listing {
    use crate::kernel::kernel::{version_from_str, KernelRelease, LinuxVersion};
    use reqwest;
    use scraper::{Html, Selector};
    use std::cmp::Ordering;

    async fn fetch_kernel_versions(major: u32) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("https://kernel.org/pub/linux/kernel/v{}.x/", major);
        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let document = Html::parse_document(&body);
        let selector = Selector::parse("a").unwrap();

        let versions = document
            .select(&selector)
            .filter_map(|element| {
                let href = element.value().attr("href")?;

                if href.ends_with(".tar.xz") || href.ends_with(".tar.gz") {
                    Some(href.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(versions)
    }

    fn is_newer_version(version: &LinuxVersion, reference: &LinuxVersion) -> Ordering {
        match version.major.cmp(&reference.major) {
            Ordering::Equal => match version.minor.cmp(&reference.minor) {
                Ordering::Equal => version.patch.cmp(&reference.patch),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }

    fn is_version_compatible(
        version: &LinuxVersion,
        last_compatible_version: &LinuxVersion,
    ) -> bool {
        return version.major <= last_compatible_version.major
            && (version.major != last_compatible_version.major
                || version.minor <= last_compatible_version.minor);
    }

    pub async fn get_last_compatible_kernel_release(
        last_compatible_version: LinuxVersion,
    ) -> Option<KernelRelease> {
        if let Ok(versions) = fetch_kernel_versions(last_compatible_version.major).await {
            let mut kernel_releases: Vec<KernelRelease> = versions
                .iter()
                .filter_map(|filename| {
                    if let Ok(version) = version_from_str(filename) {
                        if is_version_compatible(&version, &last_compatible_version) {
                            return Some(KernelRelease {
                                version: Some(version),
                                source: filename.clone(),
                            });
                        }
                    }

                    None
                })
                .collect();

            kernel_releases.sort_by(|a, b| {
                is_newer_version(a.version.as_ref().unwrap(), b.version.as_ref().unwrap())
            });

            kernel_releases.last().map(|release| KernelRelease {
                source: format!(
                    "https://kernel.org/pub/linux/kernel/v{}.x/{}",
                    release.version.as_ref().unwrap().major,
                    release.source
                ),
                version: release.version.clone(),
            })
        } else {
            println!("Failed to fetch latest kernel versions.");
            None
        }
    }
}
