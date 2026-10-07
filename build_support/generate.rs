use super::ContentIndex;
use std::collections::BTreeMap;

fn literal(text: &str) -> String {
    format!("{text:?}")
}
fn directory_symbol(index: usize) -> String {
    if index == 0 {
        "ROOT_DIRECTORY".to_owned()
    } else {
        format!("DIRECTORY_{index}")
    }
}
fn generate_articles(index: &ContentIndex, code: &mut String) {
    code.push_str(&format!(
        "static POST_DATA: [Post; {}] = [\n",
        index.articles.len()
    ));
    for (number, article) in index.articles.iter().enumerate() {
        let source = format!("{}.md", article.path.trim_end_matches('/'));
        code.push_str(&format!(
            "Post {{ path: {}, parent_path: {}, title: {}, summary: {}, date: {}, iso_date: {}, category: {}, reading_minutes: {}, index: {}, rendered: LazyLock::new(|| render_content_markdown({}, {})) }},\n",
            literal(&article.path), literal(&article.parent_path), literal(&article.title),
            literal(&article.summary), literal(&article.date), literal(&article.iso_date),
            literal(&article.category), article.reading_minutes, literal(&format!("{:02}", number + 1)),
            literal(&article.body), literal(&source),
        ));
    }
    code.push_str("];\n");
    code.push_str("pub(crate) static POSTS: &[Post] = &POST_DATA;\n");
    code.push_str("static POST_PATHS: &[(&str, &Post)] = &[\n");
    let mut path_indices: Vec<_> = (0..index.articles.len()).collect();
    path_indices.sort_by_key(|&id| &index.articles[id].path);
    for id in path_indices {
        code.push_str(&format!(
            "({}, &POSTS[{id}]),\n",
            literal(&index.articles[id].path)
        ));
    }
    code.push_str("];\n");
}
fn generate_directories(index: &ContentIndex, code: &mut String) {
    let directory_ids: BTreeMap<_, _> = index
        .directories
        .iter()
        .enumerate()
        .map(|(id, directory)| (directory.path.as_str(), id))
        .collect();
    let article_ids: BTreeMap<_, _> = index
        .articles
        .iter()
        .enumerate()
        .map(|(id, article)| (article.path.as_str(), id))
        .collect();
    for (id, directory) in index.directories.iter().enumerate() {
        let parent = directory
            .parent_path
            .as_ref()
            .map_or("None".to_owned(), |path| format!("Some({})", literal(path)));
        let source = format!("{}/_index.md", directory.path.trim_end_matches('/'));
        code.push_str(&format!(
            "pub(crate) static {}: Directory = Directory {{ id: {id}, path: {}, parent_path: {parent}, title: {}, entries: &[\n",
            directory_symbol(id), literal(&directory.path), literal(&directory.title),
        ));
        for child in index.entries.get(&directory.path).into_iter().flatten() {
            if let Some(child_id) = directory_ids.get(child.as_str()) {
                code.push_str(&format!(
                    "ContentEntry::Directory(&{}),\n",
                    directory_symbol(*child_id)
                ));
            } else if let Some(child_id) = article_ids.get(child.as_str()) {
                code.push_str(&format!("ContentEntry::Article(&POSTS[{child_id}]),\n"));
            }
        }
        code.push_str(&format!(
            "], rendered: LazyLock::new(|| render_content_markdown({}, {})) }};\n",
            literal(&directory.body),
            literal(&source),
        ));
    }
    code.push_str("pub(crate) static DIRECTORIES: &[&Directory] = &[\n");
    let mut path_indices: Vec<_> = (0..index.directories.len()).collect();
    path_indices.sort_by_key(|&id| &index.directories[id].path);
    for id in path_indices {
        code.push_str(&format!("&{},\n", directory_symbol(id)));
    }
    code.push_str("];\n");
}
pub(crate) fn generate(index: &ContentIndex) -> String {
    let mut code = String::new();
    generate_articles(index, &mut code);
    generate_directories(index, &mut code);
    code
}
