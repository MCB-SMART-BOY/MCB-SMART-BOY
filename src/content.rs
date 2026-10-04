use std::sync::LazyLock;

use crate::markdown::{RenderedMarkdown, render_markdown};

pub(crate) struct Post {
    pub(crate) slug: &'static str,
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
        // Compiled articles are immutable; share one render across body and all navigation surfaces.
        &self.rendered
    }
}

pub(crate) struct Book {
    pub(crate) slug: &'static str,
    pub(crate) title: &'static str,
    pub(crate) introduction: &'static str,
    pub(crate) chapters: &'static [Chapter],
}

pub(crate) struct Chapter {
    pub(crate) slug: &'static str,
    pub(crate) title: &'static str,
    pub(crate) post_slugs: &'static [&'static str],
}

pub(crate) const BOOKS: &[Book] = &[Book {
    slug: "demo-notes",
    title: "探索笔记（示例合集）",
    introduction: "这本示例合集收录系统设计、Rust 网页与学习记录三个方向的短文。可以先选择章节，再进入文章并按段落阅读；这些文章用于演示阅读流程，并非已经完成的连续教程。",
    chapters: &[
        Chapter {
            slug: "systems",
            title: "系统设计",
            post_slugs: &["small-systems"],
        },
        Chapter {
            slug: "rust-web",
            title: "Rust 网页",
            post_slugs: &["rust-web-notes"],
        },
        Chapter {
            slug: "learning",
            title: "学习记录",
            post_slugs: &["notes-on-learning"],
        },
    ],
}];

pub(crate) fn find_post(slug: &str) -> Option<&'static Post> {
    POSTS.iter().find(|post| post.slug == slug)
}

pub(crate) fn find_book(slug: &str) -> Option<&'static Book> {
    BOOKS.iter().find(|book| book.slug == slug)
}

pub(crate) fn find_chapter(
    book_slug: &str,
    chapter_slug: &str,
) -> Option<(&'static Book, &'static Chapter)> {
    let book = find_book(book_slug)?;
    let chapter = book
        .chapters
        .iter()
        .find(|chapter| chapter.slug == chapter_slug)?;
    Some((book, chapter))
}

pub(crate) fn find_post_location(slug: &str) -> Option<(&'static Book, &'static Chapter)> {
    BOOKS.iter().find_map(|book| {
        book.chapters
            .iter()
            .find(|chapter| chapter.post_slugs.contains(&slug))
            .map(|chapter| (book, chapter))
    })
}

// Demonstration content only; these are not presented as the author's published work.
pub(crate) static POSTS: [Post; 3] = [
    Post {
        slug: "small-systems",
        title: "把系统做小，是一种工程能力",
        summary: "从问题本身出发，去掉不需要的层，给复杂度一个进入系统的理由。",
        date: "10.03 / 2026",
        iso_date: "2026-10-03",
        category: "SYSTEMS",
        reading_minutes: 3,
        index: "01",
        rendered: LazyLock::new(|| {
            render_markdown(include_str!("../content/posts/small-systems.md"))
        }),
    },
    Post {
        slug: "rust-web-notes",
        title: "用 Rust 写一个能读的网页",
        summary: "服务端生成内容、保留原生导航，先让文字在任何设备上都可以被读到。",
        date: "09.28 / 2026",
        iso_date: "2026-09-28",
        category: "RUST / WEB",
        reading_minutes: 4,
        index: "02",
        rendered: LazyLock::new(|| {
            render_markdown(include_str!("../content/posts/rust-web-notes.md"))
        }),
    },
    Post {
        slug: "notes-on-learning",
        title: "保持一份公开的实验记录",
        summary: "记录假设、过程和偏差，而不只是漂亮的最终结果。",
        date: "09.12 / 2026",
        iso_date: "2026-09-12",
        category: "FIELD NOTES",
        reading_minutes: 2,
        index: "03",
        rendered: LazyLock::new(|| {
            render_markdown(include_str!("../content/posts/notes-on-learning.md"))
        }),
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn book_references_are_unique_and_resolve_to_one_article_location() {
        let post_slugs = POSTS.iter().map(|post| post.slug).collect::<HashSet<_>>();
        assert_eq!(post_slugs.len(), POSTS.len());

        let book_slugs = BOOKS.iter().map(|book| book.slug).collect::<HashSet<_>>();
        assert_eq!(book_slugs.len(), BOOKS.len());

        for book in BOOKS {
            let chapter_slugs = book
                .chapters
                .iter()
                .map(|chapter| chapter.slug)
                .collect::<HashSet<_>>();
            assert_eq!(chapter_slugs.len(), book.chapters.len());

            for chapter in book.chapters {
                let mut post_slugs = HashSet::new();
                for slug in chapter.post_slugs {
                    assert!(
                        post_slugs.insert(*slug),
                        "duplicate article reference: {slug}"
                    );
                    assert!(
                        find_post(slug).is_some(),
                        "unknown article reference: {slug}"
                    );
                    assert_eq!(
                        find_post_location(slug).map(|(found_book, found_chapter)| {
                            (found_book.slug, found_chapter.slug)
                        }),
                        Some((book.slug, chapter.slug))
                    );
                }
            }
        }
    }

    #[test]
    fn each_article_has_at_most_one_book_chapter_location() {
        for post in &POSTS {
            let locations = BOOKS
                .iter()
                .flat_map(|book| {
                    book.chapters.iter().filter_map(move |chapter| {
                        chapter
                            .post_slugs
                            .contains(&post.slug)
                            .then_some((book.slug, chapter.slug))
                    })
                })
                .collect::<Vec<_>>();
            assert!(
                locations.len() <= 1,
                "article has duplicate location: {}",
                post.slug
            );
        }
    }
}
