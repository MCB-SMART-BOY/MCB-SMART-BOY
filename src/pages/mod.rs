mod about;
mod article;
mod directory;
mod focus;
mod home;
mod not_found;
mod post_list;

use crate::{
    content::ROOT_DIRECTORY,
    reading::{ReadingRoute, resolve_reading_route},
};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use home::HomePage;
pub use not_found::NotFoundPage;

// All URLs share the same route boundary so the exported 404 can hydrate at any missing URL.
#[component]
pub(crate) fn SiteContent() -> impl IntoView {
    let pathname = use_location().pathname;
    move || {
        if pathname.get() == "/" {
            view! { <HomePage/> }.into_any()
        } else {
            view! { <ReadingContentPage/> }.into_any()
        }
    }
}

#[component]
fn ReadingContentPage() -> impl IntoView {
    let pathname = use_location().pathname;
    move || match resolve_reading_route(&pathname.get()) {
        Some(ReadingRoute::Index) => {
            view! { <directory::DirectoryContent directory=&ROOT_DIRECTORY is_root=true/> }
                .into_any()
        }
        Some(ReadingRoute::Directory(directory)) => {
            view! { <directory::DirectoryContent directory is_root=false/> }.into_any()
        }
        Some(ReadingRoute::Article(post)) => view! { <article::ArticlePage post/> }.into_any(),
        _ => view! { <NotFoundPage/> }.into_any(),
    }
}
