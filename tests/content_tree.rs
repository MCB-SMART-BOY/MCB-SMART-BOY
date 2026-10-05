#[path = "../build_support/mod.rs"]
mod build_support;

use build_support::load_content;
use std::{fs, path::Path};
use tempfile::TempDir;

fn put(root: &Path, relative: &str, source: &str) {
    let file = root.join(relative);
    fs::create_dir_all(file.parent().expect("fixture path has a parent"))
        .expect("create fixture directory");
    fs::write(file, source).expect("write fixture content");
}
fn fixture() -> TempDir {
    let root = TempDir::new().expect("create temporary content tree");
    put(
        root.path(),
        "_index.md",
        "---\ntitle: Writing\n---\n\nBrowse content.\n",
    );
    root
}
fn section(root: &Path, path: &str, title: &str, weight: &str) {
    put(
        root,
        &format!("{path}/_index.md"),
        &format!("---\ntitle: {title}\n{weight}---\n"),
    );
}
fn article(root: &Path, path: &str, title: &str, date: &str, weight: &str, body: &str) {
    put(
        root,
        path,
        &format!(
            "---\ntitle: {title}\nsummary: Details\ndate: '{date}'\nreading_minutes: 2\n{weight}---\n\n{body}"
        ),
    );
}
fn entry_paths(index: &build_support::ContentIndex, path: &str) -> Vec<String> {
    index.entries.get(path).cloned().unwrap_or_default()
}

#[test]
fn content_tree_new_nested_and_direct_article_appear_in_parent_order() {
    let root = fixture();
    section(root.path(), "book", "Book", "weight: 4\n");
    section(root.path(), "book/deep", "Deep", "weight: 2\n");
    article(
        root.path(),
        "book/intro.md",
        "Intro",
        "2026-10-03",
        "weight: 1\n",
        "Hello",
    );
    article(
        root.path(),
        "book/deep/final.md",
        "Final",
        "2026-10-02",
        "",
        "Done",
    );
    let index = load_content(root.path()).expect("load a nested mixed-content tree");
    assert_eq!(
        entry_paths(&index, "/writing/book"),
        ["/writing/book/intro", "/writing/book/deep"]
    );
    assert_eq!(
        entry_paths(&index, "/writing/book/deep"),
        ["/writing/book/deep/final"]
    );
    assert_eq!(
        index.directories[2].parent_path.as_deref(),
        Some("/writing/book")
    );
    assert_eq!(
        index
            .articles
            .iter()
            .find(|post| post.path == "/writing/book/intro")
            .expect("article exists")
            .body,
        "\nHello"
    );
    assert_eq!(index.directories[0].body, "\nBrowse content.\n");
}

#[test]
fn content_tree_generation_is_identical_for_different_creation_order() {
    let first = fixture();
    let second = fixture();
    article(first.path(), "zeta.md", "Zeta", "2026-10-02", "", "Z");
    article(first.path(), "alpha.md", "Alpha", "2026-10-03", "", "A");
    article(second.path(), "alpha.md", "Alpha", "2026-10-03", "", "A");
    article(second.path(), "zeta.md", "Zeta", "2026-10-02", "", "Z");
    let first_output = build_support::generate_content(first.path()).expect("compile first tree");
    let second_output =
        build_support::generate_content(second.path()).expect("compile second tree");
    assert_eq!(first_output, second_output);
}

#[test]
fn content_tree_empty_section_remains_navigable() {
    let root = fixture();
    section(root.path(), "empty", "Empty", "");
    let index = load_content(root.path()).expect("load empty directory");
    assert_eq!(entry_paths(&index, "/writing"), ["/writing/empty"]);
    assert!(entry_paths(&index, "/writing/empty").is_empty());
}

#[test]
fn content_tree_moving_article_changes_parent_and_canonical_url() {
    let root = fixture();
    section(root.path(), "first", "First", "");
    section(root.path(), "second", "Second", "");
    article(
        root.path(),
        "first/story.md",
        "Story",
        "2026-10-03",
        "",
        "Body",
    );
    let previous = load_content(root.path()).expect("load initial tree");
    assert_eq!(
        entry_paths(&previous, "/writing/first"),
        ["/writing/first/story"]
    );
    fs::rename(
        root.path().join("first/story.md"),
        root.path().join("second/story.md"),
    )
    .expect("move fixture");
    let updated = load_content(root.path()).expect("reload moved article");
    assert!(entry_paths(&updated, "/writing/first").is_empty());
    assert_eq!(
        entry_paths(&updated, "/writing/second"),
        ["/writing/second/story"]
    );
    assert!(
        updated
            .articles
            .iter()
            .all(|post| post.path != "/writing/first/story")
    );
}

#[test]
fn content_tree_weight_ties_and_latest_dates_are_deterministic() {
    let root = fixture();
    section(root.path(), "zeta", "Zeta", "weight: 1\n");
    section(root.path(), "alpha", "Alpha", "weight: 1\n");
    article(root.path(), "z-plain.md", "Later", "2026-09-12", "", "");
    article(
        root.path(),
        "a-weighted.md",
        "Latest",
        "2026-10-03",
        "weight: 1\n",
        "",
    );
    article(
        root.path(),
        "another.md",
        "Latest Too",
        "2026-10-03",
        "",
        "",
    );
    let index = load_content(root.path()).expect("sort tree");
    assert_eq!(
        entry_paths(&index, "/writing"),
        [
            "/writing/a-weighted",
            "/writing/alpha",
            "/writing/zeta",
            "/writing/another",
            "/writing/z-plain",
        ]
    );
    assert_eq!(
        index
            .articles
            .iter()
            .map(|post| post.path.as_str())
            .collect::<Vec<_>>(),
        [
            "/writing/a-weighted",
            "/writing/another",
            "/writing/z-plain",
        ]
    );
}
#[test]
fn content_tree_editing_front_matter_changes_title_weight_and_date() {
    let root = fixture();
    article(root.path(), "alpha.md", "First", "2024-02-29", "", "One");
    article(root.path(), "beta.md", "Second", "2023-02-28", "", "Two");
    let before = load_content(root.path()).expect("read initial metadata");
    assert_eq!(before.articles[0].title, "First");
    article(
        root.path(),
        "beta.md",
        "Updated",
        "2026-10-03",
        "weight: 0\n",
        "Two",
    );
    let after = load_content(root.path()).expect("re-read changed metadata");
    assert_eq!(after.articles[0].title, "Updated");
    assert_eq!(after.articles[0].date, "10.03 / 2026");
    assert_eq!(
        entry_paths(&after, "/writing"),
        ["/writing/beta", "/writing/alpha"]
    );
}

#[test]
fn content_tree_missing_index_and_duplicate_url_fail_with_file_context() {
    let root = fixture();
    article(
        root.path(),
        "orphan/post.md",
        "Orphan",
        "2026-10-03",
        "",
        "",
    );
    let error = load_content(root.path())
        .err()
        .expect("missing folder index fails")
        .to_string();
    assert!(error.contains("orphan/_index.md"), "{error}");
    section(root.path(), "orphan", "Orphan", "");
    article(root.path(), "orphan.md", "Collision", "2026-10-03", "", "");
    let error = load_content(root.path())
        .err()
        .expect("URL conflict fails")
        .to_string();
    assert!(
        error.contains("orphan.md") && error.contains("orphan/_index.md"),
        "{error}"
    );
}

#[test]
fn content_tree_bad_metadata_rejects_unknown_duplicate_and_invalid_values() {
    let valid = "title: Example\nsummary: Details\ndate: '2026-10-03'\nreading_minutes: 2\n";
    for (field, fields) in [
        ("permalink", format!("{valid}permalink: /custom\n")),
        ("duplicate title", format!("{valid}title: Again\n")),
        ("date", valid.replace("2026-10-03", "2026-02-29")),
        (
            "reading_minutes",
            valid.replace("reading_minutes: 2", "reading_minutes: 0"),
        ),
        ("weight", format!("{valid}weight: -1\n")),
        (
            "tag",
            valid.replace("title: Example", "title: !mystery Value"),
        ),
        (
            "anchor",
            valid.replace("title: Example", "title: &alias Value"),
        ),
        ("unknown key", format!("{valid}slug: custom\n")),
    ] {
        let root = fixture();
        put(
            root.path(),
            "invalid.md",
            &format!("---\n{fields}---\nBody"),
        );
        let error = load_content(root.path())
            .err()
            .expect("invalid metadata must fail")
            .to_string();
        assert!(error.contains("invalid.md"), "{field}: {error}");
    }
}

#[test]
fn content_tree_front_matter_document_end_rejects_trailing_yaml_errors() {
    for (path, fields) in [
        ("_index.md", "title: Writing\n"),
        (
            "invalid.md",
            "title: Article\nsummary: Details\ndate: '2026-10-03'\nreading_minutes: 2\n",
        ),
    ] {
        let root = fixture();
        put(
            root.path(),
            path,
            &format!("---\n{fields}...\n@ invalid YAML\n---\nBody"),
        );
        let error = load_content(root.path())
            .err()
            .expect("trailing YAML must fail");
        assert!(error.to_string().contains(path), "{error}");
        assert!(
            std::error::Error::source(&error).is_some(),
            "keep the YAML cause"
        );
    }
}

#[test]
fn content_tree_relative_markdown_links_resolve_and_missing_targets_fail() {
    let root = fixture();
    section(root.path(), "collection", "Collection", "");
    article(
        root.path(),
        "collection/target.md",
        "Target",
        "2026-10-03",
        "",
        "Content",
    );
    article(
        root.path(),
        "collection/source.md",
        "Source",
        "2026-10-02",
        "",
        "[target](target.md#section-1) [directory](_index.md)",
    );
    load_content(root.path()).expect("valid article and directory relative links");
    article(
        root.path(),
        "collection/source.md",
        "Source",
        "2026-10-02",
        "",
        "[missing](unknown.md)",
    );
    let error = load_content(root.path())
        .err()
        .expect("broken relative link fails")
        .to_string();
    assert!(
        error.contains("source.md") && error.contains("unknown.md"),
        "{error}"
    );
}

#[test]
fn content_tree_article_suffix_is_stripped_once() {
    let root = fixture();
    article(
        root.path(),
        "note.md.md",
        "Note",
        "2026-10-03",
        "",
        "[self](note.md.md)",
    );
    let index = load_content(root.path()).expect("preserve the rest of the filename");
    assert_eq!(entry_paths(&index, "/writing"), ["/writing/note.md"]);
}

#[test]
fn content_tree_external_markdown_urls_are_not_local_targets() {
    let root = fixture();
    article(
        root.path(),
        "source.md",
        "Source",
        "2026-10-03",
        "",
        "[source](https://example.org/README.md) [raw](https://example.org/README.md?raw=true)",
    );
    load_content(root.path()).expect("external Markdown URLs do not require local files");
}

#[test]
fn content_tree_relative_link_escape_and_unsafe_paths_fail() {
    for link in ["../../outside.md", "target%2e.md", "../missing.md"] {
        let root = fixture();
        article(
            root.path(),
            "source.md",
            "Source",
            "2026-10-03",
            "",
            &format!("[link]({link})"),
        );
        let error = load_content(root.path())
            .err()
            .expect("invalid link fails")
            .to_string();
        assert!(
            error.contains("source.md") && error.contains(link),
            "{error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn content_tree_symbolic_links_cannot_include_outside_content() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    let outside = TempDir::new().expect("create external tree");
    article(
        outside.path(),
        "secret.md",
        "Outside",
        "2026-10-03",
        "",
        "Outside",
    );
    symlink(
        outside.path().join("secret.md"),
        root.path().join("outside.md"),
    )
    .expect("link outside file");
    let error = load_content(root.path())
        .err()
        .expect("symbolic link fails")
        .to_string();
    assert!(
        error.contains("outside.md") && error.contains("symbolic"),
        "{error}"
    );
}

#[test]
fn content_tree_front_matter_size_and_file_limit_fail_early() {
    let root = fixture();
    let huge_metadata = format!("---\ntitle: {}\n---\n", "a".repeat(17 * 1024));
    put(root.path(), "huge.md", &huge_metadata);
    let error = load_content(root.path())
        .err()
        .expect("large YAML fails")
        .to_string();
    assert!(
        error.contains("front matter") && error.contains("huge.md"),
        "{error}"
    );
    put(
        root.path(),
        "huge.md",
        &format!(
            "---\ntitle: Test\nsummary: Summary\ndate: '2026-10-03'\nreading_minutes: 1\n---\n{}",
            "a".repeat(2 * 1024 * 1024)
        ),
    );
    let error = load_content(root.path())
        .err()
        .expect("large Markdown fails")
        .to_string();
    assert!(
        error.contains("file") && error.contains("huge.md"),
        "{error}"
    );
}
