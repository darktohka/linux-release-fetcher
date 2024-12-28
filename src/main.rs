mod kernel;
mod listing;
mod zfs;

use crate::kernel::kernel::{get_kernel_data, CachedKernelData};
use crate::zfs::zfs::{get_zfs_data, CachedZFSMeta};
use axum::{extract::State, response::Redirect, routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    kernel_cache: Arc<RwLock<CachedKernelData>>,
    zfs_cache: Arc<RwLock<CachedZFSMeta>>,
}

async fn mainline_handler(State(cache): State<AppState>) -> Result<Redirect, String> {
    let data = get_kernel_data(cache.kernel_cache).await;

    if let Some(mainline) = data.mainline {
        Ok(Redirect::temporary(&mainline.source))
    } else if let Some(stable) = data.stable {
        Ok(Redirect::temporary(&stable.source))
    } else {
        Err("Mainline kernel data not available".to_string())
    }
}

async fn stable_handler(State(cache): State<AppState>) -> Result<Redirect, String> {
    let data = get_kernel_data(cache.kernel_cache).await;

    if let Some(stable) = data.stable {
        Ok(Redirect::temporary(&stable.source))
    } else {
        Err("Stable kernel data not available".to_string())
    }
}

async fn zfs_stable_handler(State(cache): State<AppState>) -> Result<Redirect, String> {
    let data = get_zfs_data(cache.zfs_cache).await;

    if let Some(stable) = data.stable {
        Ok(Redirect::temporary(&stable.source))
    } else {
        Err("Stable ZFS data not available".to_string())
    }
}

async fn next_handler(State(cache): State<AppState>) -> Result<Redirect, String> {
    let data = get_kernel_data(cache.kernel_cache).await;

    if let Some(next) = data.next {
        Ok(Redirect::temporary(&next.source))
    } else {
        Err("Next kernel data not available".to_string())
    }
}

#[tokio::main]
async fn main() {
    let cache = Arc::new(RwLock::new(CachedKernelData::default()));
    let zfs_cache = Arc::new(RwLock::new(CachedZFSMeta::default()));
    let app_state = AppState {
        kernel_cache: cache,
        zfs_cache: zfs_cache,
    };

    let app = Router::new()
        .route("/kernel/mainline", get(mainline_handler))
        .route("/kernel/stable", get(stable_handler))
        .route("/kernel/next", get(next_handler))
        .route("/kernel/stable/zfs", get(zfs_stable_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(":::3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
