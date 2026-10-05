use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use crate::{
    content::{ContentEntry, Directory, ROOT_DIRECTORY, find_directory},
    locale::Locale,
    reading::{ReadingRoute, resolve_reading_route},
};

use super::reading_navigation::is_unmodified_click;

fn is_directory_ancestor(path: &str, directory: &Directory) -> bool {
    let mut parent_path = match resolve_reading_route(path) {
        Some(ReadingRoute::Index) => Some(ROOT_DIRECTORY.path),
        Some(ReadingRoute::Directory(current)) => Some(current.path),
        Some(ReadingRoute::Article(post)) => Some(post.parent_path),
        None => None,
    };
    while let Some(current_path) = parent_path {
        if current_path == directory.path {
            return true;
        }
        parent_path = find_directory(current_path).and_then(|current| current.parent_path);
    }
    false
}

#[component]
fn TreeLink(
    href: &'static str,
    title: &'static str,
    pathname: Memo<String>,
    depth: usize,
) -> impl IntoView {
    let is_active = Memo::new(move |_| pathname.with(|path| path == href));
    view! {
        <A href=href exact=true attr:style=format!("--tree-depth: {depth}")
            attr:class=move || if is_active.get() {
                "sidebar-link reading-tree-link is-active"
            } else {
                "sidebar-link reading-tree-link"
            }>
            <span lang="zh-CN">{title}</span>
        </A>
    }
}

#[component]
fn BranchLink(
    directory: &'static Directory,
    pathname: Memo<String>,
    is_hydrated: RwSignal<bool>,
    is_expanded: RwSignal<bool>,
    controls: String,
    depth: usize,
) -> impl IntoView {
    let is_active = Memo::new(move |_| pathname.with(|path| path == directory.path));
    view! {
        <A href=directory.path exact=true attr:data-reading-branch=""
            attr:style=format!("--tree-depth: {depth}")
            attr:class=move || if is_active.get() {
                "sidebar-link reading-tree-link is-active"
            } else {
                "sidebar-link reading-tree-link"
            }
            attr:aria-controls=controls
            attr:aria-expanded=move || (!is_hydrated.get() || is_expanded.get()).to_string()
            on:click=move |event| {
                if !is_hydrated.get_untracked() || !is_unmodified_click(&event) {
                    return;
                }
                if is_active.get_untracked() {
                    event.prevent_default();
                    is_expanded.update(|expanded| *expanded = !*expanded);
                } else {
                    is_expanded.set(true);
                }
            }>
            <span lang="zh-CN">{directory.title}</span>
            <span aria-hidden="true" class="reading-tree-chevron" hidden=move || !is_hydrated.get()></span>
        </A>
    }
}

#[component]
fn DirectoryBranch(
    directory: &'static Directory,
    pathname: Memo<String>,
    is_hydrated: RwSignal<bool>,
    id_prefix: &'static str,
    depth: usize,
    is_first_top_directory: bool,
) -> impl IntoView {
    let is_expanded = RwSignal::new(
        is_first_top_directory
            || pathname.with_untracked(|path| is_directory_ancestor(path, directory)),
    );
    Effect::new(move |_| {
        if pathname.with(|path| is_directory_ancestor(path, directory)) {
            is_expanded.set(true);
        }
    });
    let is_current_branch = move || pathname.with(|path| is_directory_ancestor(path, directory));
    let children_id = format!("reading-{id_prefix}-directory-{}", directory.id);
    view! {
        <li class="reading-tree-item">
            <div class="reading-tree-row" class:is-current-branch=is_current_branch>
                <BranchLink directory pathname is_hydrated is_expanded controls=children_id.clone() depth/>
            </div>
            <ul id=children_id class="reading-tree-list reading-tree-children"
                hidden=move || is_hydrated.get() && !is_expanded.get()>
                <DirectoryItems directory pathname is_hydrated id_prefix depth=depth + 1/>
            </ul>
        </li>
    }
}

#[component]
fn DirectoryItems(
    directory: &'static Directory,
    pathname: Memo<String>,
    is_hydrated: RwSignal<bool>,
    id_prefix: &'static str,
    depth: usize,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let first_directory_id = if depth == 0 {
        directory.entries.iter().find_map(|entry| match *entry {
            ContentEntry::Directory(child) => Some(child.id),
            ContentEntry::Article(_) => None,
        })
    } else {
        None
    };
    view! {
        {directory.entries.is_empty().then(|| view! {
            <li class="reading-tree-empty">{move || locale.get().select("暂无内容", "No content yet")}</li>
        })}
        {directory.entries.iter().map(|entry| match *entry {
            ContentEntry::Directory(child) => view! {
                <DirectoryBranch directory=child pathname is_hydrated id_prefix depth
                    is_first_top_directory=first_directory_id == Some(child.id)/>
            }.into_any(),
            ContentEntry::Article(post) => view! {
                <li><TreeLink href=post.path title=post.title pathname depth/></li>
            }.into_any(),
        }).collect_view()}
    }
}

#[component]
pub(super) fn ReadingTree(is_hydrated: RwSignal<bool>, id_prefix: &'static str) -> impl IntoView {
    let pathname = use_location().pathname;
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <nav class="reading-tree sidebar-nav" aria-label=move || locale.get().select("文章目录", "Writing directory")>
            <ul class="reading-tree-list">
                <DirectoryItems directory=&ROOT_DIRECTORY pathname is_hydrated id_prefix depth=0/>
            </ul>
        </nav>
    }
}
