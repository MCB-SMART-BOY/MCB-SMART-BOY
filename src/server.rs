use axum::Router;
use leptos::prelude::LeptosOptions;
use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};

use crate::app::{App, shell};

pub fn build_app(options: LeptosOptions) -> Router {
    let routes = generate_route_list(App);
    Router::new()
        .leptos_routes(&options, routes, {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(file_and_error_handler(shell))
        .with_state(options)
}

#[cfg(test)]
mod tests {
    use super::build_app;
    use crate::content::{BOOKS, find_post};
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header::COOKIE},
    };
    use leptos::prelude::get_configuration;
    use tower::ServiceExt;

    const MAX_TEST_RESPONSE_BYTES: usize = 1024 * 1024;

    async fn request_page_with_cookie(path: &str, cookie: Option<&str>) -> (StatusCode, String) {
        let options = get_configuration(Some("Cargo.toml"))
            .expect("valid test configuration")
            .leptos_options;
        let mut builder = Request::builder().uri(path);
        if let Some(cookie) = cookie {
            builder = builder.header(COOKIE, cookie);
        }
        let request = builder.body(Body::empty()).expect("valid URI");
        let response = build_app(options)
            .oneshot(request)
            .await
            .expect("route responds");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), MAX_TEST_RESPONSE_BYTES)
            .await
            .expect("bounded page body");
        (
            status,
            String::from_utf8(bytes.to_vec()).expect("UTF-8 page"),
        )
    }
    async fn request_page(path: &str) -> (StatusCode, String) {
        request_page_with_cookie(path, None).await
    }

    fn breadcrumb_markup(body: &str) -> &str {
        let (_, nav_content) = body
            .split_once("class=\"breadcrumbs\"")
            .expect("page has breadcrumb navigation");
        nav_content
            .split_once("</nav>")
            .expect("breadcrumb navigation closes")
            .0
    }

    fn main_markup(body: &str) -> &str {
        let (_, main_content) = body.split_once("<main ").expect("page has main content");
        main_content
            .split_once("</main>")
            .expect("main content closes")
            .0
    }

    #[tokio::test]
    async fn archive_lists_demo_book_as_real_route() {
        let (status, body) = request_page("/writing").await;
        assert_eq!(status, StatusCode::OK);
        let main = main_markup(&body);
        assert!(main.contains("href=\"/writing/books/demo-notes\""));
        assert!(main.contains("探索笔记（示例合集）"));
        assert!(main.contains(BOOKS[0].introduction));
        for chapter_slug in ["systems", "rust-web", "learning"] {
            assert!(main.contains(&format!(
                "href=\"/writing/books/demo-notes/chapters/{chapter_slug}\""
            )));
        }
        assert!(!main.contains("href=\"/writing/small-systems\""));
        assert_eq!(
            breadcrumb_markup(&body)
                .matches("aria-current=\"page\"")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn book_route_lists_chapter_pages_not_articles() {
        let (status, body) = request_page("/writing/books/demo-notes").await;
        assert_eq!(status, StatusCode::OK);
        let main = main_markup(&body);
        assert!(main.contains(BOOKS[0].introduction));
        for chapter_slug in ["systems", "rust-web", "learning"] {
            assert!(main.contains(&format!(
                "href=\"/writing/books/demo-notes/chapters/{chapter_slug}\""
            )));
        }
        for post_slug in ["small-systems", "rust-web-notes", "notes-on-learning"] {
            assert!(!main.contains(&format!("href=\"/writing/{post_slug}\"")));
        }
        let crumbs = breadcrumb_markup(&body);
        assert!(crumbs.contains("href=\"/writing\""));
        assert_eq!(crumbs.matches("aria-current=\"page\"").count(), 1);
    }

    #[tokio::test]
    async fn chapter_route_lists_only_member_articles_and_parent_breadcrumbs() {
        for (chapter_slug, own_post) in [
            ("systems", "small-systems"),
            ("rust-web", "rust-web-notes"),
            ("learning", "notes-on-learning"),
        ] {
            let (status, body) = request_page(&format!(
                "/writing/books/demo-notes/chapters/{chapter_slug}"
            ))
            .await;
            assert_eq!(status, StatusCode::OK);
            let main = main_markup(&body);
            assert!(main.contains(&format!("href=\"/writing/{own_post}\"")));
            for post_slug in ["small-systems", "rust-web-notes", "notes-on-learning"] {
                if post_slug != own_post {
                    assert!(!main.contains(&format!("href=\"/writing/{post_slug}\"")));
                }
            }
            assert!(main.contains(find_post(own_post).expect("book references a post").summary));
            let crumbs = breadcrumb_markup(&body);
            assert!(crumbs.contains("href=\"/writing\""));
            assert!(crumbs.contains("href=\"/writing/books/demo-notes\""));
            assert_eq!(crumbs.matches("aria-current=\"page\"").count(), 1);
        }
    }

    #[tokio::test]
    async fn article_breadcrumbs_link_to_actual_book_and_chapter() {
        let (status, body) = request_page("/writing/small-systems").await;
        assert_eq!(status, StatusCode::OK);
        let crumbs = breadcrumb_markup(&body);
        for ancestor in [
            "/writing",
            "/writing/books/demo-notes",
            "/writing/books/demo-notes/chapters/systems",
        ] {
            assert!(crumbs.contains(&format!("href=\"{ancestor}\"")));
        }
        assert_eq!(crumbs.matches("aria-current=\"page\"").count(), 1);
        let main = main_markup(&body);
        assert!(main.contains("href=\"/writing/books/demo-notes/chapters/systems\""));
    }

    #[tokio::test]
    async fn book_unknown_slug_returns_not_found() {
        let (status, body) = request_page("/writing/books/no-such-book").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(!main_markup(&body).contains("class=\"book-chapters\""));
    }

    #[tokio::test]
    async fn chapter_invalid_book_or_chapter_returns_not_found() {
        for path in [
            "/writing/books/no-such-book/chapters/systems",
            "/writing/books/demo-notes/chapters/no-such-chapter",
        ] {
            let (status, body) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "unexpected route: {path}");
            assert!(!main_markup(&body).contains("class=\"chapter-articles\""));
        }
    }

    #[tokio::test]
    async fn nested_writing_paths_do_not_resolve_by_prefix() {
        for path in [
            "/writing/books/demo-notes/extra",
            "/writing/books/demo-notes/chapters/systems/extra",
            "/writing/small-systems/extra",
        ] {
            let (status, _) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "unexpected route: {path}");
        }
    }

    #[tokio::test]
    async fn article_english_cookie_preserves_original_and_ui_preferences() {
        let (status, body) = request_page_with_cookie(
            "/writing/small-systems",
            Some("mcb-lang=en; mcb-ui=dark:1:320"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("lang=\"en\""));
        assert!(body.contains("data-theme=\"dark\""));
        assert!(main_markup(&body).contains("Chinese original"));
        assert!(main_markup(&body).contains("把系统做小，是一种工程能力"));
    }

    #[tokio::test]
    async fn article_invalid_language_cookie_uses_chinese() {
        let (status, body) =
            request_page_with_cookie("/writing/small-systems", Some("mcb-lang=english")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("lang=\"zh-CN\""));
        assert!(main_markup(&body).contains("中文原文"));
    }

    #[tokio::test]
    async fn article_known_slug_returns_readable_html() {
        let (status, body) = request_page("/writing/small-systems").await;
        assert_eq!(status, StatusCode::OK);
        assert!(main_markup(&body).contains("把系统做小，是一种工程能力"));
        assert!(main_markup(&body).contains("一个小系统"));
    }

    #[tokio::test]
    async fn article_html_links_outline_entries_to_real_heading_ids() {
        let (status, body) = request_page("/writing/small-systems").await;

        assert_eq!(status, StatusCode::OK);
        assert!(main_markup(&body).contains("<h2 id=\"section-1\">先写出要交付的行为</h2>"));
        assert!(main_markup(&body).contains("href=\"#section-1\""));
    }

    #[tokio::test]
    async fn article_unknown_slug_returns_not_found() {
        let (status, _) = request_page("/writing/no-such-article").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn focus_separate_route_returns_focus_content() {
        let (status, body) = request_page("/focus").await;
        assert_eq!(status, StatusCode::OK);
        for heading in ["系统编程", "智能与安全", "公开记录"] {
            assert!(main_markup(&body).contains(heading));
        }
    }

    #[tokio::test]
    async fn unknown_path_returns_not_found() {
        let (status, body) = request_page("/not-present").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(main_markup(&body).contains("没有信号"));
    }
}
