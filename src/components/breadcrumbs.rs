use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    content::{ROOT_DIRECTORY, find_directory},
    locale::Locale,
    reading::{ReadingRoute, resolve_reading_route},
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

fn ancestor_item(
    label: &'static str,
    href: &'static str,
    lang: Option<&'static str>,
) -> BreadcrumbItem {
    BreadcrumbItem {
        label,
        href: Some(href),
        is_current: false,
        lang,
    }
}

fn build_directory_ancestors(mut parent_path: Option<&'static str>) -> Vec<BreadcrumbItem> {
    let mut ancestors = Vec::new();
    while let Some(path) = parent_path {
        let Some(directory) = find_directory(path) else {
            break;
        };
        ancestors.push(ancestor_item(
            directory.title,
            directory.path,
            Some("zh-CN"),
        ));
        parent_path = directory.parent_path;
    }
    ancestors.reverse();
    ancestors
}

pub(super) fn build_breadcrumbs(pathname: &str, locale: Locale) -> Vec<BreadcrumbItem> {
    let route = resolve_reading_route(pathname);
    let (parent, current) = match route {
        Some(ReadingRoute::Index) => {
            return vec![current_item(ROOT_DIRECTORY.title, Some("zh-CN"))];
        }
        Some(ReadingRoute::Directory(directory)) => (directory.parent_path, directory.title),
        Some(ReadingRoute::Article(post)) => (Some(post.parent_path), post.title),
        None => {
            let label = match pathname {
                "/" => locale.home(),
                "/focus" => locale.focus(),
                "/about" => locale.about(),
                _ => locale.not_found(),
            };
            return vec![current_item(label, None)];
        }
    };
    let mut items = build_directory_ancestors(parent);
    items.push(current_item(current, Some("zh-CN")));
    items
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
