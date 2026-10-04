use crate::content::{Book, Chapter, Post, find_book, find_chapter, find_post};

#[derive(Clone, Copy)]
pub(crate) enum ReadingRoute {
    Index,
    Book(&'static Book),
    Chapter(&'static Book, &'static Chapter),
    Article(&'static Post),
}

pub(crate) fn resolve_reading_route(path: &str) -> Option<ReadingRoute> {
    if path == "/writing" {
        return Some(ReadingRoute::Index);
    }
    if let Some(slugs) = path.strip_prefix("/writing/books/") {
        if let Some((book_slug, chapter_slug)) = slugs.split_once("/chapters/") {
            return find_chapter(book_slug, chapter_slug)
                .map(|(book, chapter)| ReadingRoute::Chapter(book, chapter));
        }
        return find_book(slugs).map(ReadingRoute::Book);
    }
    path.strip_prefix("/writing/")
        .and_then(find_post)
        .map(ReadingRoute::Article)
}

pub(crate) fn build_chapter_path(book: &Book, chapter: &Chapter) -> String {
    format!("/writing/books/{}/chapters/{}", book.slug, chapter.slug)
}
