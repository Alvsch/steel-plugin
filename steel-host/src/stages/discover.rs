use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Context;
use flate2::read::GzDecoder;
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::config::Config;

pub struct DiscoveredPlugin {
    pub root: PathBuf,
    pub config: Config,
    pub(crate) extracted: Option<TempDir>,
}

#[derive(Debug, Clone, Copy)]
enum ArchiveKind {
    TarZstd,
    TarGz,
    Zip,
}

impl ArchiveKind {
    fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?;
        let lower_name = name.to_ascii_lowercase();

        if lower_name.ends_with(".tar.zst") {
            Some(Self::TarZstd)
        } else if lower_name.ends_with(".tar.gz") {
            Some(Self::TarGz)
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            Some(Self::Zip)
        } else {
            None
        }
    }
}

pub fn discover_plugins(path: impl AsRef<Path>) -> Vec<DiscoveredPlugin> {
    let mut discovered = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir() {
            let root = entry.into_path();
            let Ok(config) = Config::from_root(&root) else {
                continue;
            };
            discovered.push(DiscoveredPlugin {
                root,
                config,
                extracted: None,
            });
        } else if entry.file_type().is_file() {
            let path = entry.into_path();
            let Some(kind) = ArchiveKind::from_path(&path) else {
                continue;
            };
            let Ok(plugin) = load_archived_plugin(&path, kind) else {
                continue;
            };
            discovered.push(plugin);
        }
    }

    discovered
}

/// Extracts a packaged plugin archive into a fresh temp directory and loads
/// its config from there.
fn load_archived_plugin(
    archive_path: &Path,
    kind: ArchiveKind,
) -> anyhow::Result<DiscoveredPlugin> {
    let temp_dir = tempfile::Builder::new()
        .prefix("steel-plugin-")
        .tempdir()
        .context("failed to create temp dir for plugin archive")?;

    extract_archive(archive_path, temp_dir.path(), kind).with_context(|| {
        format!(
            "failed to extract plugin archive {}",
            archive_path.display()
        )
    })?;

    let config = Config::from_root(temp_dir.path()).with_context(|| {
        format!(
            "no valid config.toml in plugin archive {}",
            archive_path.display()
        )
    })?;

    Ok(DiscoveredPlugin {
        root: temp_dir.path().to_path_buf(),
        config,
        extracted: Some(temp_dir),
    })
}

/// Unpacks `archive_path` (of the given `kind`) into `dest`.
fn extract_archive(archive_path: &Path, dest: &Path, kind: ArchiveKind) -> anyhow::Result<()> {
    let file = File::open(archive_path)
        .with_context(|| format!("failed to open {}", archive_path.display()))?;

    match kind {
        ArchiveKind::TarZstd => {
            let decoder =
                zstd::stream::Decoder::new(file).context("failed to init zstd decoder")?;
            tar::Archive::new(decoder)
                .unpack(dest)
                .context("failed to unpack tar.zst archive")?;
        }
        ArchiveKind::TarGz => {
            let decoder = GzDecoder::new(file);
            tar::Archive::new(decoder)
                .unpack(dest)
                .context("failed to unpack tar.gz archive")?;
        }
        ArchiveKind::Zip => {
            let mut zip = zip::ZipArchive::new(file).context("failed to open zip archive")?;
            zip.extract(dest).context("failed to unpack zip archive")?;
        }
    }

    Ok(())
}
