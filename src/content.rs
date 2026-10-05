use std::sync::LazyLock;

use crate::markdown::{RenderedMarkdown, render_content_markdown};

pub(crate) struct Post {
    pub(crate) path: &'static str,
    pub(crate) parent_path: &'static str,
    pub(crate) title: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) date: &'static str,
    pub(crate) iso_date: &'static str,
    pub(crate) category: &'static str,
    pub(crate) reading_minutes: u8,
    pub(crate) index: &'static str,
    rendered: LazyLock<RenderedMarkdown>,
}

impl Post {
    pub(crate) fn render(&'static self) -> &'static RenderedMarkdown {
        &self.rendered
    }
}

pub(crate) struct Directory {
    pub(crate) id: usize,
    pub(crate) path: &'static str,
    pub(crate) parent_path: Option<&'static str>,
    pub(crate) title: &'static str,
    pub(crate) entries: &'static [ContentEntry],
    rendered: LazyLock<RenderedMarkdown>,
}

impl Directory {
    pub(crate) fn render(&'static self) -> &'static RenderedMarkdown {
        &self.rendered
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ContentEntry {
    Directory(&'static Directory),
    Article(&'static Post),
}

include!(concat!(env!("OUT_DIR"), "/content_index.rs"));

pub(crate) fn find_directory(path: &str) -> Option<&'static Directory> {
    let index = DIRECTORIES
        .binary_search_by_key(&path, |directory| directory.path)
        .ok()?;
    Some(DIRECTORIES[index])
}

pub(crate) fn find_post(path: &str) -> Option<&'static Post> {
    let index = POST_PATHS
        .binary_search_by_key(&path, |(key, _)| *key)
        .ok()?;
    Some(POST_PATHS[index].1)
}
