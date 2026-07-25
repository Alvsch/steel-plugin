//! Packages a Steel plugin directory into a distributable archive.
//!
//! The pipeline runs in three stages:
//! 1. [`collect_files`] — walk the plugin directory and list every file.
//! 2. [`stage_files`] — read each file into memory, precompiling `.lua`/`.luau`
//!    sources to bytecode when building in release mode.
//! 3. [`write_archive`] — write the staged files out as a `.tar.zst`, `.tar.gz`,
//!    or `.zip` archive, depending on [`PackageFormat`].

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use mlua::chunk::Compiler;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

/// Archive format to package a plugin into.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PackageFormat {
    #[value(name = "tar.zst")]
    TarZstd,
    #[value(name = "tar.gz")]
    TarGz,
    Zip,
}

impl PackageFormat {
    /// File extension used for the output archive, e.g. `"tar.zst"`.
    const fn extension(&self) -> &'static str {
        match self {
            PackageFormat::TarZstd => "tar.zst",
            PackageFormat::TarGz => "tar.gz",
            PackageFormat::Zip => "zip",
        }
    }
}

/// Options controlling how a plugin directory is packaged.
#[derive(Debug)]
pub struct PackageOptions {
    /// Root directory of the plugin to package.
    pub input: PathBuf,
    /// Directory the resulting archive is written into.
    pub output: PathBuf,
    /// Archive format to produce.
    pub format: PackageFormat,
    /// When `true`, `.lua`/`.luau` sources are compiled to optimized,
    /// debug-info-stripped bytecode instead of being packaged as source.
    pub release: bool,
}

/// A single file staged for archiving: its path relative to the plugin root,
/// and its final bytes (source, or compiled bytecode for release builds).
struct StagedFile {
    rel_path: PathBuf,
    bytes: Vec<u8>,
}

/// Packages the plugin described by `opts`, returning the path to the
/// resulting archive.
///
/// # Errors
/// Returns an error if the input directory can't be walked, any file can't
/// be read or compiled, or the archive can't be written.
pub fn package_plugin(opts: PackageOptions) -> Result<PathBuf> {
    ensure_config_present(&opts.input)?;

    let files = collect_files(&opts.input)?;
    let staged = stage_files(&opts.input, &files, opts.release)?;
    let archive_path = archive_output_path(&opts);

    write_archive(&staged, &archive_path, &opts.format)?;

    Ok(archive_path)
}

/// Bails out immediately if `input` doesn't contain a `config.toml`, before
/// any walking, compiling, or archiving is attempted.
fn ensure_config_present(input: &Path) -> Result<()> {
    let config_path = input.join("config.toml");

    anyhow::ensure!(
        config_path.is_file(),
        "no config.toml found in plugin directory {}",
        input.display()
    );

    Ok(())
}

/// Stage 1: recursively lists every regular file under `input`.
///
/// Unlike a naive walk, directory-traversal errors (e.g. a broken symlink or
/// a permissions failure partway through the tree) are surfaced as an error
/// rather than silently skipped, so a partially-packaged plugin never goes
/// unnoticed.
fn collect_files(input: &Path) -> Result<Vec<PathBuf>> {
    WalkDir::new(input)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) if entry.file_type().is_file() => Some(Ok(entry.into_path())),
            Ok(_) => None, // directories, symlinks, etc. — nothing to stage
            Err(err) => Some(Err(err)),
        })
        .collect::<walkdir::Result<_>>()
        .with_context(|| format!("failed to walk plugin directory {}", input.display()))
}

/// Stage 2: turns every file into its final on-disk representation.
///
/// `.lua`/`.luau` files are precompiled to bytecode when `release` is set;
/// everything else passes through unchanged.
fn stage_files(root: &Path, files: &[PathBuf], release: bool) -> Result<Vec<StagedFile>> {
    let compiler = if release {
        Compiler::new().set_optimization_level(2).set_debug_level(0)
    } else {
        Compiler::new()
    };

    files
        .iter()
        .map(|path| stage_one_file(root, path, &compiler))
        .collect()
}

/// Stages a single file relative to `root`, compiling it first if it's a Luau
/// source file.
fn stage_one_file(root: &Path, path: &Path, compiler: &Compiler) -> Result<StagedFile> {
    let mut rel_path = path
        .strip_prefix(root)
        .with_context(|| {
            format!(
                "{} is not under plugin root {}",
                path.display(),
                root.display()
            )
        })?
        .to_path_buf();

    let bytes = if is_luau_source(path) {
        rel_path.set_extension("luac");
        compile_luau(path, compiler)?
    } else {
        fs::read(path).with_context(|| format!("failed to read {}", path.display()))?
    };

    Ok(StagedFile { rel_path, bytes })
}

/// Whether `path` is a Luau/Lua source file that should be compiled rather
/// than copied verbatim.
fn is_luau_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("lua" | "luau")
    )
}

/// Precompiles a single `.lua`/`.luau` source file to bytecode.
fn compile_luau(path: &Path, compiler: &Compiler) -> Result<Vec<u8>> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    compiler
        .compile(&source)
        .with_context(|| format!("failed to compile {}", path.display()))
}

/// Computes where the final archive file should live, named after the input
/// directory (e.g. `output/my-plugin.tar.zst`).
fn archive_output_path(opts: &PackageOptions) -> PathBuf {
    let stem = opts
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("plugin");

    opts.output
        .join(format!("{stem}.{}", opts.format.extension()))
}

/// Stage 3: writes staged files into the requested archive format.
fn write_archive(staged: &[StagedFile], archive_path: &Path, format: &PackageFormat) -> Result<()> {
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output dir {}", parent.display()))?;
    }

    match format {
        PackageFormat::TarZstd => {
            let file = create_archive_file(archive_path)?;
            let encoder = zstd::stream::Encoder::new(file, 19)?.auto_finish();
            write_tar_archive(encoder, staged)
        }
        PackageFormat::TarGz => {
            let file = create_archive_file(archive_path)?;
            let encoder = GzEncoder::new(file, flate2::Compression::best());
            write_tar_archive(encoder, staged)
        }
        PackageFormat::Zip => write_zip(staged, archive_path),
    }
}

/// Creates (or truncates) the archive file at `archive_path`.
fn create_archive_file(archive_path: &Path) -> Result<fs::File> {
    fs::File::create(archive_path)
        .with_context(|| format!("failed to create {}", archive_path.display()))
}

/// Writes `staged` into a tar archive over the given `encoder`, used for both
/// the `tar.zst` and `tar.gz` formats (they differ only in compression).
fn write_tar_archive<W: Write>(encoder: W, staged: &[StagedFile]) -> Result<()> {
    let mut builder = tar::Builder::new(encoder);
    append_staged_to_tar(&mut builder, staged)?;
    builder.finish().context("failed to finalize tar archive")
}

fn append_staged_to_tar<W: Write>(
    builder: &mut tar::Builder<W>,
    staged: &[StagedFile],
) -> Result<()> {
    for file in staged {
        let mut header = tar::Header::new_gnu();
        header.set_size(file.bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        builder
            .append_data(&mut header, &file.rel_path, file.bytes.as_slice())
            .with_context(|| format!("failed to add {} to archive", file.rel_path.display()))?;
    }
    Ok(())
}

fn write_zip(staged: &[StagedFile], archive_path: &Path) -> Result<()> {
    let file = create_archive_file(archive_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for file_entry in staged {
        let name = file_entry
            .rel_path
            .to_str()
            .with_context(|| format!("non-utf8 path {}", file_entry.rel_path.display()))?;

        zip.start_file(name, options)
            .with_context(|| format!("failed to start zip entry {name:?}"))?;
        zip.write_all(&file_entry.bytes)
            .with_context(|| format!("failed to write zip entry {name:?}"))?;
    }

    zip.finish().context("failed to finalize zip archive")?;
    Ok(())
}
