pub mod zfs {
    use reqwest;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::{Duration, Instant};

    use crate::kernel::kernel::{KernelRelease, LinuxVersion};
    use crate::listing::listing::get_last_compatible_kernel_release;

    #[derive(Default, Clone)]
    pub struct ZFSMeta {
        pub stable: Option<KernelRelease>,
    }

    pub struct CachedZFSMeta {
        data: ZFSMeta,
        last_updated: Instant,
    }

    impl Default for CachedZFSMeta {
        fn default() -> Self {
            CachedZFSMeta {
                data: ZFSMeta::default(),
                last_updated: Instant::now()
                    .checked_sub(Duration::from_secs(120))
                    .unwrap(),
            }
        }
    }

    impl CachedZFSMeta {
        fn is_expired(&self) -> bool {
            self.last_updated.elapsed() > Duration::from_secs(60)
        }
    }

    async fn fetch_zfs_data() -> ZFSMeta {
        const ZFS_META_URL: &str =
            "https://raw.githubusercontent.com/openzfs/zfs/refs/heads/master/META";
        let response = reqwest::get(ZFS_META_URL).await;

        if let Ok(resp) = response {
            if let Ok(data) = resp.text().await {
                if let Some(line) = data.lines().find(|line| line.starts_with("Linux-Maximum:")) {
                    let version_str = line.split(':').nth(1).unwrap_or("").trim();
                    let version_parts: Vec<&str> = version_str.split('.').collect();
                    if version_parts.len() == 2 {
                        let major = version_parts[0].parse::<u32>().unwrap_or(0);
                        let minor = version_parts[1].parse::<u32>().unwrap_or(0);
                        return ZFSMeta {
                            stable: get_last_compatible_kernel_release(LinuxVersion {
                                major,
                                minor,
                                patch: 0,
                            })
                            .await,
                        };
                    }
                }
            }
        }

        ZFSMeta::default()
    }

    pub async fn get_zfs_data(cache: Arc<RwLock<CachedZFSMeta>>) -> ZFSMeta {
        let mut cache = cache.write().await;
        if cache.is_expired() {
            cache.data = fetch_zfs_data().await;
            cache.last_updated = Instant::now();
        }
        cache.data.clone()
    }
}
