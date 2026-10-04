use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    content::{Book, Chapter, Post, find_post_location},
    locale::Locale,
    reading::{ReadingRoute, build_chapter_path, resolve_reading_route},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BreadcrumbItem {
    label: &'static str,
    href: Option<String>,
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

fn ancestor_item(label: &'static str, href: String, lang: Option<&'static str>) -> BreadcrumbItem {
    BreadcrumbItem {
        label,
        href: Some(href),
        is_current: false,
        lang,
    }
}

fn build_book_breadcrumbs(book: &Book, locale: Locale) -> Vec<BreadcrumbItem> {
    vec![
        ancestor_item(locale.writing(), "/writing".to_owned(), None),
        current_item(book.title, Some("zh-CN")),
    ]
}

fn build_chapter_breadcrumbs(
    book: &Book,
    chapter: &Chapter,
    locale: Locale,
) -> Vec<BreadcrumbItem> {
    vec![
        ancestor_item(locale.writing(), "/writing".to_owned(), None),
        ancestor_item(
            book.title,
            format!("/writing/books/{}", book.slug),
            Some("zh-CN"),
        ),
        current_item(chapter.title, Some("zh-CN")),
    ]
}

fn build_article_breadcrumbs(post: &Post, locale: Locale) -> Vec<BreadcrumbItem> {
    let mut items = Vec::with_capacity(4);
    items.push(ancestor_item(locale.writing(), "/writing".to_owned(), None));
    if let Some((book, chapter)) = find_post_location(post.slug) {
        items.push(ancestor_item(
            book.title,
            format!("/writing/books/{}", book.slug),
            Some("zh-CN"),
        ));
        items.push(ancestor_item(
            chapter.title,
            build_chapter_path(book, chapter),
            Some("zh-CN"),
        ));
    }
    items.push(current_item(post.title, Some("zh-CN")));
    items
}

pub(super) fn build_breadcrumbs(pathname: &str, locale: Locale) -> Vec<BreadcrumbItem> {
    let label = match pathname {
        "/" => locale.home(),
        "/focus" => locale.focus(),
        "/about" => locale.about(),
        _ => match resolve_reading_route(pathname) {
            Some(ReadingRoute::Index) => locale.writing(),
            Some(ReadingRoute::Book(book)) => return build_book_breadcrumbs(book, locale),
            Some(ReadingRoute::Chapter(book, chapter)) => {
                return build_chapter_breadcrumbs(book, chapter, locale);
            }
            Some(ReadingRoute::Article(post)) => return build_article_breadcrumbs(post, locale),
            None => locale.not_found(),
        },
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
                {move || items.get().into_iter().enumerate().map(|(index, item)| view! {
                    <li lang=item.lang aria-current=if item.is_current { Some("page") } else { None }>
                        {(index > 0).then(|| view! { <span class="breadcrumb-separator" aria-hidden="true">"›"</span> })}
                        {match item.href {
                            Some(href) => view! { <a href=href><span class="breadcrumb-label" title=item.label>{item.label}</span></a> }.into_any(),
                            None => view! { <span class="breadcrumb-label" title=item.label>{item.label}</span> }.into_any(),
                        }}
                    </li>
                }).collect_view()}
            </ol>
        </nav>
    }
}
