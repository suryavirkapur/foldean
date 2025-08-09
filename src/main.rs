use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use dirs_next::download_dir;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Cross-platform Downloads organizer.
///
/// Safe by default: runs in dry-run mode unless --apply is passed.
#[derive(Parser, Debug)]
#[command(name = "foldean", about = "Organize your Downloads into tidy folders", version)]
struct Cli {
    /// Directory to organize (defaults to OS Downloads directory)
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Actually move files instead of printing the plan
    #[arg(short = 'y', long = "apply", action = ArgAction::SetTrue)]
    apply: bool,

    /// Include hidden files
    #[arg(long, action = ArgAction::SetTrue)]
    include_hidden: bool,

    /// Maximum depth to scan (0 means only the target directory)
    #[arg(long, default_value_t = 0)]
    depth: usize,
}

/// Folder categories keyed by folder name with supported extensions (lowercase, no dot)
static CATEGORY_EXTENSIONS: Lazy<BTreeMap<&'static str, BTreeSet<&'static str>>> = Lazy::new(|| {
    let mut map: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();
    let mut add = |category: &'static str, exts: &[&'static str]| {
        map.entry(category)
            .or_insert_with(BTreeSet::new)
            .extend(exts.iter().copied());
    };

    // Documents
    add(
        "Documents",
        &["pdf", "doc", "docx", "rtf", "txt", "md", "markdown", "odt", "oxps"],
    );
    add("Sheets", &["xls", "xlsx", "csv", "ods"]); // spreadsheets & data
    add("Slides", &["ppt", "pptx", "key"]); // presentations

    // Media
    add("Images", &["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "tiff", "heic"]);
    add("Audio", &["mp3", "wav", "m4a", "flac", "aac", "ogg"]);
    add("Videos", &["mp4", "mov", "mkv", "avi", "webm"]);

    // Code & data
    add(
        "Code",
        &["c", "cpp", "h", "hpp", "rs", "py", "js", "ts", "tsx", "java", "go", "rb", "sh", "yaml", "yml", "json", "toml"],
    );
    add("Books", &["epub", "mobi", "azw", "azw3", "pdf"]);

    // Archives & installers
    add("Archives", &["zip", "rar", "7z", "tar", "gz", "bz2", "xz"]);
    add("Installer", &["dmg", "pkg", "msi", "exe", "deb", "rpm", "appimage", "app"]);

    // Design/graphics
    add("Design", &["psd", "ai", "xd", "fig", "sketch"]);

    map
});

fn main() -> Result<()> {
    let cli = Cli::parse();

    let target_dir = cli
        .dir
        .or_else(|| download_dir())
        .context("Could not resolve Downloads directory. Pass --dir explicitly.")?;

    let plan = build_plan(&target_dir, cli.depth, cli.include_hidden)?;

    if plan.is_empty() {
        println!("Nothing to organize in {}", target_dir.display());
        return Ok(());
    }

    println!("Planned moves ({}):", plan.len());
    for (from, to) in &plan {
        println!("  {} -> {}", from.display(), to.display());
    }

    if cli.apply {
        apply_moves(plan)?;
        println!("Done.");
    } else {
        println!("Dry run. Pass --apply to move files.");
    }

    Ok(())
}

fn build_plan(dir: &Path, depth: usize, include_hidden: bool) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut moves: Vec<(PathBuf, PathBuf)> = Vec::new();

    for entry in fs::read_dir(dir).with_context(|| format!("Reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if !include_hidden && file_name_str.starts_with('.') {
            continue;
        }

        // Skip common temporary Office files
        if file_name_str.starts_with("~$") {
            continue;
        }

        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            // Recurse if allowed depth > 0, but do not move folders at root level
            if depth > 0 {
                let child_moves = build_plan(&path, depth - 1, include_hidden)?;
                moves.extend(child_moves);
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| String::from(""));

        let category = match_category(ext.as_str());
        let category = category.unwrap_or("Others");

        let dest_dir = dir.join(category);
        let dest_path = unique_destination(&dest_dir, path.file_name().unwrap());
        if path != dest_path {
            moves.push((path, dest_path));
        }
    }

    Ok(moves)
}

fn match_category(ext: &str) -> Option<&'static str> {
    if ext.is_empty() {
        return None;
    }
    for (category, extensions) in CATEGORY_EXTENSIONS.iter() {
        if extensions.contains(ext) {
            return Some(category);
        }
    }
    None
}

fn unique_destination(dest_dir: &Path, file_name: &OsStr) -> PathBuf {
    let mut candidate = dest_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("file");
    let ext = Path::new(file_name)
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| format!(".{}", s))
        .unwrap_or_default();

    let mut counter: u32 = 1;
    loop {
        let new_name = format!("{} ({}){}", stem, counter, ext);
        candidate = dest_dir.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

fn apply_moves(moves: Vec<(PathBuf, PathBuf)>) -> Result<()> {
    for (from, to) in moves {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).with_context(|| format!("Create {}", parent.display()))?;
        }

        // Use rename first; if cross-filesystem, fallback to copy + remove
        match fs::rename(&from, &to) {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::CrossesDevices => {
                fs::copy(&from, &to)
                    .with_context(|| format!("Copy {} -> {}", from.display(), to.display()))?;
                fs::remove_file(&from).with_context(|| format!("Remove {}", from.display()))?;
            }
            Err(err) => return Err(err).with_context(|| format!("Move {} -> {}", from.display(), to.display())),
        }
    }
    Ok(())
}
