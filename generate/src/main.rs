//! Turn a Lucide release into `src/icon.rs` and `src/meta.rs`.
//!
//! ```text
//! cargo run -p generate -- 1.40.0            # download the tag from GitHub
//! cargo run -p generate -- 1.40.0 --from-dir path/to/lucide/icons
//! ```
//!
//! Output is sorted by icon name, so a second run on the same tag writes the
//! same bytes.

mod emit;
mod naming;
mod svg;

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

/// One icon as Lucide ships it.
pub struct IconSrc {
    pub name: String,
    pub body: String,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
}

const USAGE: &str =
    "usage: cargo run -p generate -- <lucide-tag> [--from-dir <icons dir>] [--out <crate dir>]";

fn main() -> Result<()> {
    let mut tag = None;
    let mut from_dir = None;
    let mut out_dir = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from-dir" => from_dir = Some(PathBuf::from(args.next().context(USAGE)?)),
            "--out" => out_dir = Some(PathBuf::from(args.next().context(USAGE)?)),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            other if other.starts_with('-') => bail!("unknown flag {other}\n{USAGE}"),
            other => tag = Some(other.to_string()),
        }
    }
    let tag = tag.context(USAGE)?;
    let crate_dir = out_dir.unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the generator sits inside the crate")
            .to_path_buf()
    });

    let files = match &from_dir {
        Some(dir) => {
            println!("reading lucide {tag} from {}", dir.display());
            read_dir(dir)?
        }
        None => {
            let url =
                format!("https://github.com/lucide-icons/lucide/archive/refs/tags/{tag}.tar.gz");
            println!("downloading {url}");
            read_archive(&download(&url)?)?
        }
    };

    let icons = parse_icons(&files)?;
    println!("{} icons", icons.len());

    let svg_attrs = uniform_svg_attrs(&files)?;

    let src = crate_dir.join("src");
    write_if_changed(
        &src.join("icon.rs"),
        &formatted("icon", &emit::icon_rs(&tag, &svg_attrs, &icons))?,
    )?;
    write_if_changed(
        &src.join("meta.rs"),
        &formatted("meta", &emit::meta_rs(&tag, &icons))?,
    )?;
    set_version(&crate_dir.join("Cargo.toml"), &tag)?;
    Ok(())
}

/// The bytes of one icon's two files, keyed by icon name.
#[derive(Default)]
struct Files {
    svg: Option<String>,
    json: Option<String>,
}

fn download(url: &str) -> Result<Vec<u8>> {
    let mut response = ureq::get(url).call().context("GitHub said no")?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(128 * 1024 * 1024)
        .read_to_vec()
        .context("reading the archive")?;
    println!("{} bytes", bytes.len());
    Ok(bytes)
}

fn read_archive(tarball: &[u8]) -> Result<BTreeMap<String, Files>> {
    let mut out: BTreeMap<String, Files> = BTreeMap::new();
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let mut parts = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string());
        let (_root, dir, file) = (parts.next(), parts.next(), parts.next());
        if dir.as_deref() != Some("icons") || parts.next().is_some() {
            continue;
        }
        let Some(file) = file else { continue };
        let mut text = String::new();
        entry.read_to_string(&mut text)?;
        match file.rsplit_once('.') {
            Some((name, "svg")) => out.entry(name.to_string()).or_default().svg = Some(text),
            Some((name, "json")) => out.entry(name.to_string()).or_default().json = Some(text),
            _ => continue,
        }
    }
    ensure!(!out.is_empty(), "the archive has no icons/ directory");
    Ok(out)
}

fn read_dir(dir: &Path) -> Result<BTreeMap<String, Files>> {
    let mut out: BTreeMap<String, Files> = BTreeMap::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        let Some(name) = path.file_stem().map(|s| s.to_string_lossy().to_string()) else {
            continue;
        };
        let text = || std::fs::read_to_string(&path);
        match path.extension().and_then(|e| e.to_str()) {
            Some("svg") => out.entry(name).or_default().svg = Some(text()?),
            Some("json") => out.entry(name).or_default().json = Some(text()?),
            _ => continue,
        }
    }
    ensure!(!out.is_empty(), "{} has no icons", dir.display());
    Ok(out)
}

fn parse_icons(files: &BTreeMap<String, Files>) -> Result<Vec<IconSrc>> {
    let mut icons = Vec::with_capacity(files.len());
    let mut variants: BTreeMap<String, String> = BTreeMap::new();
    for (name, file) in files {
        let source = file
            .svg
            .as_ref()
            .with_context(|| format!("{name} has no .svg"))?;
        let meta = file
            .json
            .as_ref()
            .with_context(|| format!("{name} has no .json"))?;
        let parsed = svg::parse(source).with_context(|| format!("parsing {name}.svg"))?;
        let meta: serde_json::Value =
            serde_json::from_str(meta).with_context(|| format!("parsing {name}.json"))?;

        let variant = naming::pascal(name);
        if let Some(other) = variants.insert(variant.clone(), name.clone()) {
            bail!("{name} and {other} both want the variant {variant}");
        }

        icons.push(IconSrc {
            name: name.clone(),
            body: parsed.body,
            tags: strings(&meta, "tags"),
            categories: strings(&meta, "categories"),
        });
    }
    Ok(icons)
}

fn strings(value: &serde_json::Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|i| i.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Every Lucide icon carries the same `<svg>` attributes. Prove it, so the
/// crate can hold one copy.
fn uniform_svg_attrs(files: &BTreeMap<String, Files>) -> Result<String> {
    let mut seen: Option<(String, String)> = None;
    for (name, file) in files {
        let Some(source) = &file.svg else { continue };
        let attrs = svg::parse(source)?.attrs;
        match &seen {
            None => seen = Some((name.clone(), attrs)),
            Some((first, first_attrs)) => ensure!(
                *first_attrs == attrs,
                "{name} carries different <svg> attributes than {first}:\n  {attrs}\n  {first_attrs}"
            ),
        }
    }
    Ok(seen.context("no icons")?.1)
}

fn write_if_changed(path: &Path, contents: &str) -> Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|old| old == contents) {
        println!("{} unchanged", path.display());
        return Ok(());
    }
    std::fs::write(path, contents).with_context(|| format!("writing {}", path.display()))?;
    println!("{} written ({} bytes)", path.display(), contents.len());
    Ok(())
}

/// The crate version is the Lucide version.
fn set_version(manifest: &Path, tag: &str) -> Result<()> {
    let text = std::fs::read_to_string(manifest)?;
    let mut out = String::with_capacity(text.len());
    let mut in_package = false;
    let mut done = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
        }
        if in_package && !done && trimmed.starts_with("version = ") {
            out.push_str(&format!("version = \"{tag}\"\n"));
            done = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    ensure!(done, "no version in [package] of {}", manifest.display());
    if out != text {
        std::fs::write(manifest, &out)?;
        println!("{} set to version {tag}", manifest.display());
    }
    Ok(())
}

/// Run the emitted code past rustfmt before comparing it with what is on disk,
/// so `cargo fmt --all --check` has nothing to say and a second run on the same
/// tag writes nothing.
fn formatted(name: &str, contents: &str) -> Result<String> {
    let path = std::env::temp_dir().join(format!("typed-lucide-{}-{name}.rs", std::process::id()));
    std::fs::write(&path, contents)?;
    let status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(&path)
        .status()
        .context("running rustfmt — is it installed?")?;
    ensure!(status.success(), "rustfmt refused the generated {name}.rs");
    let out = std::fs::read_to_string(&path)?;
    std::fs::remove_file(&path)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icons() -> BTreeMap<String, Files> {
        let svg = |body: &str| {
            format!(
                "<svg\n  xmlns=\"http://www.w3.org/2000/svg\"\n  width=\"24\"\n  height=\"24\"\n  \
                 viewBox=\"0 0 24 24\"\n  fill=\"none\"\n  stroke=\"currentColor\"\n  \
                 stroke-width=\"2\"\n  stroke-linecap=\"round\"\n  stroke-linejoin=\"round\"\n>\n  \
                 {body}\n</svg>\n"
            )
        };
        let json = |tags: &str, categories: &str| {
            format!("{{\"tags\": [{tags}], \"categories\": [{categories}]}}")
        };
        // Inserted out of order on purpose: the output must not care.
        let mut files = BTreeMap::new();
        for (name, body, tags, cats) in [
            (
                "grid-2x2",
                "<rect x=\"1\" y=\"1\" width=\"4\" height=\"4\" />",
                "\"grid\"",
                "\"layout\"",
            ),
            (
                "a-arrow-down",
                "<path d=\"m14 12 4 4 4-4\" />",
                "\"letter\", \"text\"",
                "\"text\"",
            ),
        ] {
            files.insert(
                name.to_string(),
                Files {
                    svg: Some(svg(body)),
                    json: Some(json(tags, cats)),
                },
            );
        }
        files
    }

    #[test]
    fn the_same_tag_emits_the_same_bytes() {
        let files = icons();
        let first = parse_icons(&files).unwrap();
        let second = parse_icons(&files).unwrap();
        let attrs = uniform_svg_attrs(&files).unwrap();

        assert_eq!(
            emit::icon_rs("1.40.0", &attrs, &first),
            emit::icon_rs("1.40.0", &attrs, &second)
        );
        assert_eq!(
            emit::meta_rs("1.40.0", &first),
            emit::meta_rs("1.40.0", &second)
        );
    }

    #[test]
    fn the_output_is_sorted_by_name_whatever_order_the_archive_gave() {
        let icons = parse_icons(&icons()).unwrap();
        let names: Vec<&str> = icons.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["a-arrow-down", "grid-2x2"]);

        let file = emit::icon_rs("1.40.0", "", &icons);
        assert!(file.starts_with(
            "// GENERATED by typed-lucide/generate from lucide 1.40.0 — do not edit\n"
        ));
        assert!(file.contains("    AArrowDown,\n"));
        assert!(file.contains("    Grid2x2,\n"));
        assert!(
            file.find("AArrowDown").unwrap() < file.find("Grid2x2").unwrap(),
            "the enum is in name order"
        );
        assert!(
            file.contains(r#""<path d=\"m14 12 4 4 4-4\"/>""#),
            "the body keeps lucide's markup"
        );
    }

    #[test]
    fn a_tag_becomes_the_crate_version() {
        let dir = std::env::temp_dir().join("typed-lucide-generate-test");
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = dir.join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"typed-lucide\"\nversion = \"1.39.0\"\n\n[dependencies]\nx = { version = \"1.0.0\" }\n",
        )
        .unwrap();

        set_version(&manifest, "1.40.0").unwrap();

        let text = std::fs::read_to_string(&manifest).unwrap();
        assert!(text.contains("version = \"1.40.0\""));
        assert!(
            text.contains("x = { version = \"1.0.0\" }"),
            "only [package] moves"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
