use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use crate::{
    content::{POSTS, Post},
    locale::Locale,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BreadcrumbItem {
    label: &'static str,
    href: Option<&'static str>,
    is_current: bool,
    lang: Option<&'static str>,
}

fn current_item(label: &'static str, lang: Option<&'static str>) -> BreadcrumbItem {
    BreadcrumbItem {
        label,
        href: None,
        is_current: true,
        lang,
    }
}

fn build_article_breadcrumbs(post: &Post, locale: Locale) -> Vec<BreadcrumbItem> {
    let mut items = Vec::with_capacity(post.trail.len() + 2);
    items.push(BreadcrumbItem {
        label: locale.writing(),
        href: Some("/writing"),
        is_current: false,
        lang: None,
    });
    items.extend(
        post.trail
            .iter()
            .filter(|part| !part.trim().is_empty())
            .map(|part| BreadcrumbItem {
                label: part,
                href: None,
                is_current: false,
                lang: Some("zh-CN"),
            }),
    );
    items.push(current_item(post.title, Some("zh-CN")));
    items
}

pub(super) fn build_breadcrumbs(pathname: &str, locale: Locale) -> Vec<BreadcrumbItem> {
    let label = match pathname {
        "/" => locale.home(),
        "/writing" => locale.writing(),
        "/focus" => locale.focus(),
        "/about" => locale.about(),
        _ => {
            if let Some(post) = pathname
                .strip_prefix("/writing/")
                .and_then(|slug| POSTS.iter().find(|post| post.slug == slug))
            {
                return build_article_breadcrumbs(post, locale);
            }
            locale.not_found()
        }
    };
    vec![current_item(label, None)]
}

#[component]
pub(super) fn Breadcrumbs() -> impl IntoView {
    let location = use_location();
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let items = Memo::new(move |_| build_breadcrumbs(&location.pathname.get(), locale.get()));
    view! {
        <nav class="breadcrumbs" tabindex="0" aria-label=move || locale.get().select("当前位置", "Current location")>
            <ol>
                {move || items.get().into_iter().map(|item| view! {
                    <li lang=item.lang aria-current=if item.is_current { Some("page") } else { None }>
                        {match item.href {
                            Some(href) => view! { <A href=href>{item.label}</A> }.into_any(),
                            None => view! { <span>{item.label}</span> }.into_any(),
                        }}
                    </li>
                }).collect_view()}
            </ol>
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breadcrumbs_known_routes_and_unknown_paths_are_readable() {
        assert_eq!(build_breadcrumbs("/", Locale::ZhCn)[0].label, "首页");
        assert_eq!(build_breadcrumbs("/focus", Locale::En)[0].label, "Focus");
        assert_eq!(
            build_breadcrumbs("/not-present", Locale::En)[0].label,
            "Page not found"
        );
        assert_eq!(
            build_breadcrumbs("/writing/missing", Locale::ZhCn)[0].label,
            "页面未找到"
        );
    }

    #[test]
    fn breadcrumbs_article_links_archive_and_preserves_original_title() {
        let items = build_breadcrumbs("/writing/small-systems", Locale::En);
        assert_eq!(items[0].href, Some("/writing"));
        assert_eq!(items[0].label, "Writing");
        assert_eq!(items[1].label, "把系统做小，是一种工程能力");
        assert_eq!(items[1].lang, Some("zh-CN"));
        assert!(items[1].is_current);
    }

    #[test]
    fn breadcrumbs_nested_trail_omits_blank_parts_without_fake_links() {
        let post = Post {
            slug: "lesson",
            title: "第一课",
            summary: "",
            date: "",
            iso_date: "",
            category: "",
            reading_minutes: 1,
            index: "",
            body: "",
            trail: &["nixos", " ", "nix教学"],
        };
        let items = build_article_breadcrumbs(&post, Locale::ZhCn);
        assert_eq!(
            items.iter().map(|item| item.label).collect::<Vec<_>>(),
            vec!["文章", "nixos", "nix教学", "第一课"]
        );
        assert!(items[1..3].iter().all(|item| item.href.is_none()));
        assert_eq!(items[3].lang, Some("zh-CN"));
    }
}
