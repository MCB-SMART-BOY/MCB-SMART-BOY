mod generate;
mod links;
mod metadata;

pub(crate) use metadata::io_error;
use metadata::{
    ArticleMeta, ContentError, DirectoryMeta, invalid, parse_metadata, read_content, require_text,
    validate_date,
};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const MAX_DIRECTORY_DEPTH: usize = 32;
const MAX_ENTRIES: usize = 10_000;

pub(crate) struct DirectoryData {
    pub(crate) path: String,
    pub(crate) parent_path: Option<String>,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) weight: Option<u32>,
    pub(crate) file: PathBuf,
}
pub(crate) struct ArticleData {
    pub(crate) path: String,
    pub(crate) parent_path: String,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) summary: String,
    pub(crate) date: String,
    pub(crate) iso_date: String,
    pub(crate) category: String,
    pub(crate) reading_minutes: u8,
    pub(crate) weight: Option<u32>,
    pub(crate) file: PathBuf,
}
pub(crate) struct ContentIndex {
    pub(crate) directories: Vec<DirectoryData>,
    pub(crate) articles: Vec<ArticleData>,
    pub(crate) entries: BTreeMap<String, Vec<String>>,
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && !segment.starts_with('.')
        && !segment.ends_with('.')
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}
fn validate_entry_name(path: &Path, name: &str) -> Result<(), ContentError> {
    if valid_segment(name) {
        Ok(())
    } else {
        Err(invalid(
            path,
            "name",
            format!("expected a URL-safe ASCII name: {name}"),
        ))
    }
}
fn list_directory(directory: &Path) -> Result<Vec<(String, PathBuf, fs::FileType)>, ContentError> {
    let entries =
        fs::read_dir(directory).map_err(|err| io_error(directory, "list directory", err))?;
    let mut sorted = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| io_error(directory, "read directory entry", err))?;
        let path = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| invalid(&path, "name", "must be UTF-8"))?;
        let kind = entry
            .file_type()
            .map_err(|err| io_error(&path, "inspect entry", err))?;
        if kind.is_symlink() {
            return Err(invalid(
                &path,
                "file",
                "symbolic links are not allowed in the content tree",
            ));
        }
        sorted.push((name, path, kind));
    }
    sorted.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(sorted)
}
fn add_directory(
    index: &mut ContentIndex,
    dir: &Path,
    path: String,
    parent: Option<String>,
) -> Result<(), ContentError> {
    let file = dir.join("_index.md");
    let (yaml, body) = read_content(&file)?;
    let meta: DirectoryMeta = parse_metadata(&file, &yaml)?;
    require_text(&file, "title", &meta.title)?;
    index.directories.push(DirectoryData {
        path,
        parent_path: parent,
        title: meta.title,
        body,
        weight: meta.weight,
        file,
    });
    Ok(())
}
fn add_article(
    index: &mut ContentIndex,
    file: &Path,
    path: String,
    parent: &str,
) -> Result<(), ContentError> {
    let (yaml, body) = read_content(file)?;
    let meta: ArticleMeta = parse_metadata(file, &yaml)?;
    require_text(file, "title", &meta.title)?;
    require_text(file, "summary", &meta.summary)?;
    let date = validate_date(file, &meta.date)?;
    if meta.reading_minutes == 0 {
        return Err(invalid(
            file,
            "reading_minutes",
            "must be positive (1..=255)",
        ));
    }
    if meta
        .category
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(invalid(
            file,
            "category",
            "when provided, must not be blank",
        ));
    }
    index.articles.push(ArticleData {
        path,
        parent_path: parent.to_owned(),
        title: meta.title,
        body,
        summary: meta.summary,
        date,
        iso_date: meta.date,
        category: meta.category.unwrap_or_default(),
        reading_minutes: meta.reading_minutes,
        weight: meta.weight,
        file: file.to_owned(),
    });
    Ok(())
}
fn scan_directory(
    index: &mut ContentIndex,
    dir: &Path,
    page: String,
    parent: Option<String>,
    depth: usize,
) -> Result<(), ContentError> {
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(invalid(dir, "depth", "too many nested content folders"));
    }
    add_directory(index, dir, page.clone(), parent)?;
    for (name, file, kind) in list_directory(dir)? {
        if name == "_index.md" {
            continue;
        }
        if kind.is_dir() {
            validate_entry_name(&file, &name)?;
            scan_directory(
                index,
                &file,
                format!("{page}{name}/"),
                Some(page.clone()),
                depth + 1,
            )?;
        } else if kind.is_file() && name.ends_with(".md") {
            let stem = file
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| invalid(&file, "name", "expected a UTF-8 Markdown filename"))?;
            validate_entry_name(&file, stem)?;
            add_article(index, &file, format!("{page}{stem}/"), &page)?;
        } else if !kind.is_file() {
            return Err(invalid(
                &file,
                "file",
                "expected ordinary file or directory",
            ));
        }
        if index.directories.len() + index.articles.len() > MAX_ENTRIES {
            return Err(invalid(&file, "content tree", "exceeds 10000 pages"));
        }
    }
    Ok(())
}
fn validate_links(
    index: &ContentIndex,
    paths: &BTreeMap<String, &Path>,
) -> Result<(), ContentError> {
    for directory in &index.directories {
        let source = format!("{}/_index.md", directory.path.trim_end_matches('/'));
        validate_body_links(&directory.file, &source, &directory.body, paths)?;
    }
    for article in &index.articles {
        let source = format!("{}.md", article.path.trim_end_matches('/'));
        validate_body_links(&article.file, &source, &article.body, paths)?;
    }
    Ok(())
}
fn validate_body_links(
    file: &Path,
    source: &str,
    body: &str,
    paths: &BTreeMap<String, &Path>,
) -> Result<(), ContentError> {
    for event in Parser::new_ext(body, Options::ENABLE_STRIKETHROUGH) {
        let Event::Start(Tag::Link { dest_url, .. }) = event else {
            continue;
        };
        let destination = links::resolve_content_link(source, &dest_url).map_err(|source| {
            ContentError::Link {
                path: file.display().to_string(),
                destination: dest_url.to_string(),
                source,
            }
        })?;
        if let Some(target) = destination {
            let page = target.split('#').next().unwrap_or_default();
            if !paths.contains_key(page) {
                return Err(invalid(
                    file,
                    "Markdown link",
                    format!("missing target {dest_url} -> {page}"),
                ));
            }
        }
    }
    Ok(())
}
fn sort_entries(index: &mut ContentIndex) -> Result<(), ContentError> {
    let mut paths = BTreeMap::new();
    let mut order = BTreeMap::new();
    for directory in &index.directories {
        if let Some(previous) = paths.insert(directory.path.clone(), directory.file.as_path()) {
            return Err(invalid(
                &directory.file,
                "URL",
                format!("{} conflicts with {}", directory.path, previous.display()),
            ));
        }
        order.insert(
            directory.path.clone(),
            (directory.weight.is_none(), directory.weight),
        );
        index.entries.entry(directory.path.clone()).or_default();
    }
    for article in &index.articles {
        if let Some(previous) = paths.insert(article.path.clone(), article.file.as_path()) {
            return Err(invalid(
                &article.file,
                "URL",
                format!("{} conflicts with {}", article.path, previous.display()),
            ));
        }
        order.insert(
            article.path.clone(),
            (article.weight.is_none(), article.weight),
        );
        index
            .entries
            .entry(article.parent_path.clone())
            .or_default()
            .push(article.path.clone());
    }
    for directory in index.directories.iter().skip(1) {
        let Some(parent) = directory.parent_path.as_ref() else {
            continue;
        };
        index
            .entries
            .entry(parent.clone())
            .or_default()
            .push(directory.path.clone());
    }
    for entries in index.entries.values_mut() {
        entries.sort_by(|a, b| order.get(a).cmp(&order.get(b)).then_with(|| a.cmp(b)));
    }
    validate_links(index, &paths)?;
    index.articles.sort_by(|a, b| {
        b.iso_date
            .cmp(&a.iso_date)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(())
}

pub(crate) fn load_content(root: &Path) -> Result<ContentIndex, ContentError> {
    let kind =
        fs::symlink_metadata(root).map_err(|err| io_error(root, "inspect content root", err))?;
    if !kind.file_type().is_dir() {
        return Err(invalid(
            root,
            "root",
            "must be an ordinary directory, not a symbolic link",
        ));
    }
    let mut index = ContentIndex {
        directories: Vec::new(),
        articles: Vec::new(),
        entries: BTreeMap::new(),
    };
    scan_directory(&mut index, root, "/writing/".to_owned(), None, 0)?;
    sort_entries(&mut index)?;
    Ok(index)
}

pub(crate) fn generate_content(root: &Path) -> Result<String, ContentError> {
    let index = load_content(root)?;
    Ok(generate::generate(&index))
}
