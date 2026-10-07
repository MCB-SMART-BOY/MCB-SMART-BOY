use crate::content::{Directory, Post, find_directory, find_post};

#[derive(Clone, Copy)]
pub(crate) enum ReadingRoute {
    Index,
    Directory(&'static Directory),
    Article(&'static Post),
}

pub(crate) fn resolve_reading_route(path: &str) -> Option<ReadingRoute> {
    if path == "/writing/" {
        return Some(ReadingRoute::Index);
    }
    if !path.starts_with("/writing/") {
        return None;
    }
    find_directory(path)
        .map(ReadingRoute::Directory)
        .or_else(|| find_post(path).map(ReadingRoute::Article))
}
