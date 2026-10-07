use axum::Router;
use leptos::prelude::LeptosOptions;
use leptos_axum::{AxumRouteListing, LeptosRoutes, file_and_error_handler};
use leptos_router::{Method, SsrMode};

use crate::{app::shell, content::page_paths};

pub fn build_app(options: LeptosOptions) -> Router {
    let routes = page_paths()
        .map(|path| AxumRouteListing::new(path.to_owned(), SsrMode::Async, [Method::Get], vec![]))
        .collect();
    let render_options = options.clone();
    Router::new()
        .leptos_routes(&options, routes, move || shell(render_options.clone()))
        .fallback(file_and_error_handler(shell))
        .with_state(options)
}

#[cfg(test)]
mod tests {
    use super::build_app;
    use crate::content::{find_directory, find_post, page_paths};
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header::COOKIE},
    };
    use leptos::prelude::get_configuration;
    use tower::ServiceExt;

    const MAX_TEST_RESPONSE_BYTES: usize = 1024 * 1024;
    const DEMO_DIRECTORY_PATH: &str = "/writing/demo-notes/";
    const SYSTEMS_DIRECTORY_PATH: &str = "/writing/demo-notes/systems/";
    const SYSTEMS_POST_PATH: &str = "/writing/demo-notes/systems/small-systems/";
    const ARTICLE_PATHS: [&str; 3] = [
        SYSTEMS_POST_PATH,
        "/writing/demo-notes/rust-web/rust-web-notes/",
        "/writing/demo-notes/learning/notes-on-learning/",
    ];

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

    fn find_element_markup<'a>(body: &'a str, marker: &str, tag: &str) -> &'a str {
        let marker_start = body.find(marker).expect("expected rendered element");
        let start = body[..marker_start]
            .rfind('<')
            .expect("element opening tag");
        let body = &body[start..];
        let opening = format!("<{tag}");
        let closing = format!("</{tag}>");
        let mut depth = 0;
        for (offset, _) in body.match_indices('<') {
            let remaining = &body[offset..];
            if remaining.starts_with(&opening)
                && remaining
                    .as_bytes()
                    .get(opening.len())
                    .is_some_and(|character| character.is_ascii_whitespace() || *character == b'>')
            {
                depth += 1;
            } else if remaining.starts_with(&closing) {
                depth -= 1;
                if depth == 0 {
                    return &body[..offset + closing.len()];
                }
            }
        }
        panic!("unclosed rendered {tag}");
    }

    fn find_active_navigation(body: &str) -> &str {
        let sidebar = find_element_markup(body, "id=\"site-sidebar\"", "aside");
        find_element_markup(sidebar, "aria-hidden=\"false\"", "div")
    }

    fn find_current_directory(body: &str) -> &str {
        find_element_markup(body, "class=\"current-directory\"", "aside")
    }

    fn collect_attribute_values<'a>(markup: &'a str, name: &str) -> Vec<&'a str> {
        markup
            .split(&format!(" {name}=\""))
            .skip(1)
            .map(|part| part.split_once('\"').expect("quoted attribute").0)
            .collect()
    }

    fn collect_link_targets(markup: &str) -> Vec<&str> {
        collect_attribute_values(markup, "href")
    }

    fn assert_current_link(markup: &str, expected_href: &str) {
        assert_eq!(
            markup.matches("aria-current=\"page\"").count(),
            1,
            "current navigation at {expected_href}: {markup}"
        );
        let current = find_element_markup(markup, "aria-current=\"page\"", "a");
        assert_eq!(collect_link_targets(current), [expected_href]);
    }

    #[tokio::test]
    async fn archive_lists_only_direct_content_children() {
        let (status, body) = request_page("/writing/").await;
        assert_eq!(status, StatusCode::OK);
        let targets = collect_link_targets(main_markup(&body));
        assert!(targets.contains(&DEMO_DIRECTORY_PATH));
        assert!(!targets.contains(&SYSTEMS_DIRECTORY_PATH));
        for path in ARTICLE_PATHS {
            assert!(!targets.contains(&path));
        }
        assert_eq!(
            breadcrumb_markup(&body)
                .matches("aria-current=\"page\"")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn directory_lists_direct_children_and_own_introduction() {
        let (status, body) = request_page(DEMO_DIRECTORY_PATH).await;
        assert_eq!(status, StatusCode::OK);
        let main = main_markup(&body);
        let directory = find_directory(DEMO_DIRECTORY_PATH).expect("fixture directory");
        assert!(main.contains(directory.render().html.as_str()));
        let targets = collect_link_targets(main);
        for name in ["systems", "rust-web", "learning"] {
            let path = format!("{DEMO_DIRECTORY_PATH}{name}/");
            assert!(targets.contains(&path.as_str()));
        }
        for path in ARTICLE_PATHS {
            assert!(!targets.contains(&path));
        }
        let crumbs = breadcrumb_markup(&body);
        assert!(collect_link_targets(crumbs).contains(&"/writing/"));
        assert_eq!(crumbs.matches("aria-current=\"page\"").count(), 1);
    }

    #[tokio::test]
    async fn directory_with_articles_links_only_local_posts_and_real_parent() {
        for (name, own_post) in ["systems", "rust-web", "learning"]
            .into_iter()
            .zip(ARTICLE_PATHS)
        {
            let (status, body) = request_page(&format!("{DEMO_DIRECTORY_PATH}{name}/")).await;
            assert_eq!(status, StatusCode::OK);
            let main = main_markup(&body);
            let targets = collect_link_targets(main);
            assert!(targets.contains(&own_post));
            for path in ARTICLE_PATHS.into_iter().filter(|path| *path != own_post) {
                assert!(!targets.contains(&path));
            }
            assert!(main.contains(find_post(own_post).expect("fixture article").summary));
            let crumbs = breadcrumb_markup(&body);
            let ancestors = collect_link_targets(crumbs);
            assert!(ancestors.contains(&"/writing/"));
            assert!(ancestors.contains(&DEMO_DIRECTORY_PATH));
            assert_eq!(crumbs.matches("aria-current=\"page\"").count(), 1);
        }
    }

    #[tokio::test]
    async fn article_breadcrumbs_and_return_link_follow_actual_directories() {
        let (status, body) = request_page(SYSTEMS_POST_PATH).await;
        assert_eq!(status, StatusCode::OK);
        let crumbs = breadcrumb_markup(&body);
        let ancestors = collect_link_targets(crumbs);
        for ancestor in ["/writing/", DEMO_DIRECTORY_PATH, SYSTEMS_DIRECTORY_PATH] {
            assert!(ancestors.contains(&ancestor));
        }
        assert_eq!(crumbs.matches("aria-current=\"page\"").count(), 1);
        assert!(collect_link_targets(main_markup(&body)).contains(&SYSTEMS_DIRECTORY_PATH));
    }

    #[tokio::test]
    async fn nonexistent_content_paths_return_not_found() {
        for path in [
            "/writing/no-such-directory/",
            "/writing/demo-notes/no-such-directory/",
            "/writing/demo-notes/systems/no-such-article/",
        ] {
            let (status, _) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "unexpected route: {path}");
        }
    }

    #[tokio::test]
    async fn no_slash_content_paths_return_not_found_instead_of_aliasing() {
        for path in [
            "/writing",
            "/writing/demo-notes",
            "/writing/demo-notes/systems/small-systems",
        ] {
            let (status, body) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "unexpected route: {path}");
            assert!(
                main_markup(&body).contains("404"),
                "missing error page at {path}"
            );
        }
    }

    #[tokio::test]
    async fn legacy_content_addresses_return_not_found_without_aliases() {
        for path in [
            "/writing/books/demo-notes/",
            "/writing/books/demo-notes/chapters/systems/",
            "/writing/books/demo-notes/chapters/rust-web/",
            "/writing/books/demo-notes/chapters/learning/",
            "/writing/small-systems/",
            "/writing/rust-web-notes/",
            "/writing/notes-on-learning/",
        ] {
            let (status, _) = request_page(path).await;
            assert_eq!(
                status,
                StatusCode::NOT_FOUND,
                "legacy route remains: {path}"
            );
        }
    }

    #[tokio::test]
    async fn nested_writing_paths_do_not_resolve_by_prefix() {
        for path in [
            "/writing/demo-notes/extra/",
            "/writing/demo-notes/systems/extra/",
            "/writing/demo-notes/systems/small-systems/extra/",
            "/writing/demo-notes/systems/small-systems.md/",
        ] {
            let (status, _) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "unexpected route: {path}");
        }
    }

    #[tokio::test]
    async fn article_english_cookie_preserves_original_and_ui_preferences() {
        let (status, body) =
            request_page_with_cookie(SYSTEMS_POST_PATH, Some("mcb-lang=en; mcb-ui=dark:1:320"))
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
            request_page_with_cookie(SYSTEMS_POST_PATH, Some("mcb-lang=english")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("lang=\"zh-CN\""));
        assert!(main_markup(&body).contains("中文原文"));
    }

    #[tokio::test]
    async fn article_content_path_returns_readable_html_without_front_matter() {
        let (status, body) = request_page(SYSTEMS_POST_PATH).await;
        assert_eq!(status, StatusCode::OK);
        assert!(main_markup(&body).contains("把系统做小，是一种工程能力"));
        assert!(main_markup(&body).contains("一个小系统"));
        assert!(!main_markup(&body).contains("reading_minutes:"));
    }

    #[tokio::test]
    async fn article_html_links_outline_entries_to_real_heading_ids() {
        let (status, body) = request_page(SYSTEMS_POST_PATH).await;

        assert_eq!(status, StatusCode::OK);
        assert!(main_markup(&body).contains("<h2 id=\"section-1\">先写出要交付的行为</h2>"));
        assert!(main_markup(&body).contains("href=\"#section-1\""));
    }

    #[tokio::test]
    async fn article_unknown_slug_returns_not_found() {
        let (status, _) = request_page("/writing/no-such-article/").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_path_returns_not_found() {
        let (status, _) = request_page("/not-present").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unknown_writing_and_root_paths_share_generic_404_navigation() {
        let (expected_status, expected_body) = request_page("/404.html").await;
        assert_eq!(expected_status, StatusCode::NOT_FOUND);
        for path in ["/not-present", "/writing/no-such-article/"] {
            let (status, body) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
            assert_eq!(main_markup(&body), main_markup(&expected_body), "{path}");
            assert_eq!(
                collect_link_targets(find_active_navigation(&body)),
                collect_link_targets(find_active_navigation(&expected_body)),
                "{path}"
            );
        }
    }

    #[tokio::test]
    async fn retired_profile_routes_return_not_found() {
        for path in ["/focus", "/about"] {
            let (status, _) = request_page(path).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        }
    }

    #[tokio::test]
    async fn reading_routes_keep_complete_tree_and_exact_current_page() {
        let expected = [
            "/",
            "/writing/",
            DEMO_DIRECTORY_PATH,
            SYSTEMS_DIRECTORY_PATH,
            SYSTEMS_POST_PATH,
            "/writing/demo-notes/rust-web/",
            "/writing/demo-notes/rust-web/rust-web-notes/",
            "/writing/demo-notes/learning/",
            "/writing/demo-notes/learning/notes-on-learning/",
        ];
        let generated = page_paths().collect::<Vec<_>>();
        assert_eq!(generated.len(), expected.len());
        assert_eq!(
            generated
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>(),
            expected.into_iter().collect()
        );
        for path in generated {
            let (status, _) = request_page(path).await;
            assert_eq!(status, StatusCode::OK, "generated route: {path}");
        }
        for path in [
            "/writing/",
            DEMO_DIRECTORY_PATH,
            "/writing/demo-notes/rust-web/",
            SYSTEMS_POST_PATH,
        ] {
            let (status, body) = request_page(path).await;
            assert_eq!(status, StatusCode::OK);
            let navigation = find_active_navigation(&body);
            assert_eq!(collect_link_targets(navigation), expected, "{path}");
            assert_current_link(navigation, path);
        }
    }

    #[tokio::test]
    async fn reading_disclosures_have_unique_targets_and_visible_ssr_children() {
        let (_, body) = request_page(SYSTEMS_POST_PATH).await;
        let ids = collect_attribute_values(&body, "id");
        let unique = ids
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), ids.len(), "duplicate element IDs");
        let scopes = [
            find_element_markup(&body, "id=\"site-sidebar\"", "aside"),
            find_element_markup(&body, "class=\"mobile-drawer\"", "dialog"),
        ];
        for scope in scopes {
            let tree = find_element_markup(scope, "class=\"reading-tree sidebar-nav\"", "nav");
            for target in collect_attribute_values(tree, "aria-controls") {
                assert!(
                    unique.contains(target),
                    "missing disclosure target: {target}"
                );
                let link = find_element_markup(tree, &format!("aria-controls=\"{target}\""), "a");
                assert!(
                    link.contains("aria-expanded=\"true\""),
                    "SSR disclosure state is false"
                );
                let targets = collect_link_targets(link);
                assert_eq!(targets.len(), 1);
                assert!(
                    find_directory(targets[0]).is_some(),
                    "branch does not open a directory"
                );
                let subtree = find_element_markup(tree, &format!("id=\"{target}\""), "ul");
                let opening = subtree.split_once('>').expect("subtree opening tag").0;
                assert!(!opening.contains(" hidden"), "SSR hides links in {target}");
            }
        }
    }

    #[tokio::test]
    async fn article_directory_links_only_to_real_page_headings() {
        let (_, body) = request_page(SYSTEMS_POST_PATH).await;
        let expected = find_post(SYSTEMS_POST_PATH)
            .expect("fixture article")
            .render()
            .headings
            .iter()
            .map(|heading| format!("{SYSTEMS_POST_PATH}#{}", heading.id))
            .collect::<Vec<_>>();
        assert_eq!(
            collect_link_targets(find_current_directory(&body)),
            expected
        );
    }

    #[tokio::test]
    async fn non_article_and_invalid_routes_omit_page_directory() {
        for path in [
            "/",
            "/focus",
            "/about",
            "/writing/",
            DEMO_DIRECTORY_PATH,
            SYSTEMS_DIRECTORY_PATH,
            "/missing",
            "/writing/no-such-article/",
            "/writing/no-such-directory/",
            "/writing/demo-notes/no-such-directory/",
        ] {
            let (_, body) = request_page(path).await;
            assert!(
                !body.contains("class=\"current-directory\""),
                "unexpected directory at {path}"
            );
            assert!(
                !body.contains("class=\"article-outline-compact\""),
                "unexpected article outline at {path}"
            );
            assert!(
                !body.contains("has-current-directory"),
                "empty directory column at {path}"
            );
        }
    }
}
