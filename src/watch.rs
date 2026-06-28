use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::{Duration, Instant};

pub fn watch_project(dir: &Path) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(dir, RecursiveMode::Recursive)?;
    
    println!("Watching directory {} for changes... (Ctrl+C to stop)", dir.display());
    
    let exclusions = ["node_modules", "target", ".git", ".venv", "onpkg_docs", "onpkg.json", "onpkg-context.md", "onpkg-context.xml", "onpkg-context.json"];
    
    let debounce_duration = Duration::from_secs(2);
    let mut pending_sync = false;
    let mut deadline = Instant::now();

    loop {
        let timeout = if pending_sync {
            let now = Instant::now();
            if now >= deadline {
                // Deadline reached! Run sync.
                println!("Syncing project manifest...");
                let path_to_sync = PathBuf::from(dir);
                if let Err(e) = crate::templates::sync_onpkg_project(&path_to_sync, None, None) {
                    tracing::warn!("Auto-sync failed: {}", e);
                } else {
                    println!("Sync complete.");
                }
                pending_sync = false;
                debounce_duration // wait standard duration next
            } else {
                deadline.duration_since(now)
            }
        } else {
            // No sync pending, block for a while
            Duration::from_secs(60)
        };

        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                let should_sync = event.paths.iter().any(|path| {
                    !path.components().any(|c| {
                        if let std::path::Component::Normal(name) = c {
                            exclusions.contains(&name.to_string_lossy().as_ref())
                        } else {
                            false
                        }
                    })
                });
                
                if should_sync {
                    if !pending_sync {
                        pending_sync = true;
                    }
                    // Reset deadline to 2 seconds from now
                    deadline = Instant::now() + debounce_duration;
                }
            }
            Ok(Err(e)) => tracing::error!("Watcher error: {}", e),
            Err(RecvTimeoutError::Timeout) => {
                // If it timed out, loop around, and the deadline check at the start will trigger the sync.
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}
