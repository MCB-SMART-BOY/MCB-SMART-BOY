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
        assert!(body.contains("Chinese original"));
        assert!(body.contains("把系统做小，是一种工程能力"));
        assert!(body.contains("Back to all posts"));
    }

    #[tokio::test]
    async fn article_invalid_language_cookie_uses_chinese() {
        let (status, body) =
            request_page_with_cookie("/writing/small-systems", Some("mcb-lang=english")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("lang=\"zh-CN\""));
        assert!(body.contains("中文原文"));
        assert!(body.contains("返回全部文章"));
    }

    #[tokio::test]
    async fn article_known_slug_returns_readable_html() {
        let (status, body) = request_page("/writing/small-systems").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("把系统做小，是一种工程能力"));
        assert!(body.contains("一个小系统"));
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
            assert!(body.contains(heading));
        }
    }

    #[tokio::test]
    async fn unknown_path_returns_not_found() {
        let (status, body) = request_page("/not-present").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("没有信号"));
    }
}
