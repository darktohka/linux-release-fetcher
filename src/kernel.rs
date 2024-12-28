pub mod kernel {
    use reqwest;
    use serde::Deserialize;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::{Duration, Instant};

    #[derive(Default, Clone)]
    pub struct LinuxVersion {
        pub major: u32,
        pub minor: u32,
        pub patch: u32,
    }

    #[derive(Default, Clone)]
    pub struct KernelRelease {
        pub version: Option<LinuxVersion>,
        pub source: String,
    }

    #[derive(Default, Clone)]
    pub struct KernelData {
        pub mainline: Option<KernelRelease>,
        pub stable: Option<KernelRelease>,
        pub next: Option<KernelRelease>,
        releases: Vec<GivenKernelRelease>,
    }

    pub struct CachedKernelData {
        data: KernelData,
        last_updated: Instant,
    }

    impl Default for CachedKernelData {
        fn default() -> Self {
            CachedKernelData {
                data: KernelData::default(),
                last_updated: Instant::now()
                    .checked_sub(Duration::from_secs(120))
                    .unwrap(),
            }
        }
    }

    #[derive(Debug, Deserialize, Clone)]
    struct GivenKernelRelease {
        moniker: String,
        version: String,
        source: Option<String>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct KernelReleases {
        releases: Vec<GivenKernelRelease>,
    }

    impl CachedKernelData {
        fn is_expired(&self) -> bool {
            self.last_updated.elapsed() > Duration::from_secs(60)
        }
    }

    pub fn version_from_str(s: &str) -> Result<LinuxVersion, &'static str> {
        let pos = s.find(|c: char| c.is_ascii_digit());
        let kernel = if let Some(pos) = pos {
            let (_, s) = s.split_at(pos);
            s
        } else {
            s
        };

        let mut kernel_split = kernel.split('.');

        let major = kernel_split
            .next()
            .ok_or("Missing major version component")?;
        let minor = kernel_split
            .next()
            .ok_or("Missing minor version component")?;
        let patch = kernel_split
            .next()
            .ok_or("Missing patch version component")?;

        let major = major.parse().map_err(|_| "Failed to parse major version")?;
        let minor = minor.parse().map_err(|_| "Failed to parse minor version")?;
        let patch = patch.parse().unwrap_or(0);

        Ok(LinuxVersion {
            major,
            minor,
            patch,
        })
    }

    async fn fetch_kernel_data() -> KernelData {
        let url = "https://kernel.org/releases.json";
        let response = reqwest::get(url).await;

        if let Ok(resp) = response {
            if let Ok(data) = resp.json::<KernelReleases>().await {
                let mut kernel_data = KernelData::default();
                kernel_data.releases = data.releases;

                kernel_data.releases.iter().for_each(|release| {
                    if release.moniker == "mainline" && !kernel_data.mainline.is_some() {
                        kernel_data.mainline = Some(KernelRelease {
                            version: version_from_str(&release.version).ok(),
                            source: release.source.clone().unwrap_or_default(),
                        });
                    } else if release.moniker == "stable" && !kernel_data.stable.is_some() {
                        kernel_data.stable = Some(KernelRelease {
                            version: version_from_str(&release.version).ok(),
                            source: release.source.clone().unwrap_or_default(),
                        });
                    } else if release.moniker == "linux-next" && !kernel_data.next.is_some() {
                        kernel_data.next = Some(KernelRelease {
                            version: None,
                            source: format!(
                                "https://git.kernel.org/pub/scm/linux/kernel/git/next/linux-next.git/snapshot/linux-next-{}.tar.gz",
                                release.version
                            ),
                        });
                    }
                });

                return kernel_data;
            }
        }

        KernelData::default()
    }

    pub async fn get_kernel_data(cache: Arc<RwLock<CachedKernelData>>) -> KernelData {
        let mut cache = cache.write().await;
        if cache.is_expired() {
            cache.data = fetch_kernel_data().await;
            cache.last_updated = Instant::now();
        }
        cache.data.clone()
    }
}
