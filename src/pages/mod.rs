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

pub use home::HomePage;
pub use not_found::NotFoundPage;

#[component]
pub fn ReadingContentPage() -> impl IntoView {
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
