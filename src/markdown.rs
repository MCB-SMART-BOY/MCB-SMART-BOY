use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

fn is_safe_destination(destination: &str) -> bool {
    destination.starts_with("https://")
        || destination.starts_with("http://")
        || destination.starts_with("mailto:")
        || (destination.starts_with('/') && !destination.starts_with("//"))
        || destination.starts_with('#')
}

fn safe_destination(destination: CowStr<'_>) -> CowStr<'_> {
    if is_safe_destination(&destination) {
        destination
    } else {
        CowStr::Borrowed("#")
    }
}

pub(crate) fn render_markdown(source: &str) -> String {
    let events = Parser::new_ext(source, Options::ENABLE_STRIKETHROUGH).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: safe_destination(dest_url),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: safe_destination(dest_url),
            title,
            id,
        }),
        other => other,
    });
    let mut rendered = String::new();
    html::push_html(&mut rendered, events);
    rendered
}

#[cfg(test)]
mod tests {
    use super::{is_safe_destination, render_markdown};

    #[test]
    fn markdown_raw_html_and_unsafe_link_are_not_executable() {
        let html = render_markdown("Hello <script>alert(1)</script> [click](javascript:alert)");
        assert!(!html.contains("<script>"));
        assert!(!html.contains("href=\"javascript:"));
        assert!(html.contains("href=\"#\""));
    }

    #[test]
    fn markdown_https_link_and_code_remain_readable() {
        let html = render_markdown("[Rust](https://www.rust-lang.org)\n\n`let x = 1;`");
        assert!(html.contains("href=\"https://www.rust-lang.org\""));
        assert!(html.contains("<code>let x = 1;</code>"));
        assert!(!is_safe_destination("//example.com"));
    }
}
