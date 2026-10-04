use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeadingEntry {
    pub(crate) id: String,
    pub(crate) text: String,
    pub(crate) level: u8,
}

pub(crate) struct RenderedMarkdown {
    pub(crate) html: String,
    pub(crate) headings: Vec<HeadingEntry>,
}
const FIRST_SECTION_NUMBER: usize = 1;

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

fn sanitize_event(event: Event<'_>) -> Event<'_> {
    match event {
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
    }
}

pub(crate) fn render_markdown(source: &str) -> RenderedMarkdown {
    let mut headings = Vec::new();
    let mut current_heading: Option<HeadingEntry> = None;
    let mut next_section = FIRST_SECTION_NUMBER;
    let events = Parser::new_ext(source, Options::ENABLE_STRIKETHROUGH).map(|event| {
        let event = sanitize_event(event);
        let event = match event {
            Event::Start(Tag::Heading {
                level,
                classes,
                attrs,
                ..
            }) => {
                let id = format!("section-{next_section}");
                next_section += 1;
                current_heading = Some(HeadingEntry {
                    id: id.clone(),
                    text: String::new(),
                    level: level as u8,
                });
                Event::Start(Tag::Heading {
                    level,
                    id: Some(CowStr::Boxed(id.into_boxed_str())),
                    classes,
                    attrs,
                })
            }
            other => other,
        };
        match &event {
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = current_heading.as_mut() {
                    heading.text.push_str(text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = current_heading.as_mut() {
                    heading.text.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(mut heading) = current_heading.take() {
                    heading.text = normalize_heading_text(&heading.text);
                    if !heading.text.is_empty() {
                        headings.push(heading);
                    }
                }
            }
            _ => {}
        }
        event
    });
    let mut html = String::new();
    html::push_html(&mut html, events);
    RenderedMarkdown { html, headings }
}

fn normalize_heading_text(text: &str) -> String {
    let mut words = text.split_whitespace();
    let mut normalized = words.next().unwrap_or_default().to_owned();
    for word in words {
        normalized.push(' ');
        normalized.push_str(word);
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{is_safe_destination, render_markdown};

    #[test]
    fn markdown_raw_html_and_unsafe_link_are_not_executable() {
        let html =
            render_markdown("Hello <script>alert(1)</script> [click](javascript:alert)").html;
        assert!(!html.contains("<script>"));
        assert!(!html.contains("href=\"javascript:"));
        assert!(html.contains("href=\"#\""));
    }

    #[test]
    fn markdown_https_link_and_code_remain_readable() {
        let html = render_markdown("[Rust](https://www.rust-lang.org)\n\n`let x = 1;`").html;
        assert!(html.contains("href=\"https://www.rust-lang.org\""));
        assert!(html.contains("<code>let x = 1;</code>"));
        assert!(!is_safe_destination("//example.com"));
    }

    #[test]
    fn markdown_headings_have_matching_unique_ids_and_preserve_structure() {
        let rendered = render_markdown(
            "# 重复\n\n## 重复\n\n#### [链接](https://example.com) `code`\n\n```md\n# not a heading\n```\n",
        );

        assert_eq!(
            rendered
                .headings
                .iter()
                .map(|heading| (heading.id.as_str(), heading.level, heading.text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("section-1", 1, "重复"),
                ("section-2", 2, "重复"),
                ("section-3", 4, "链接 code"),
            ]
        );
        assert!(rendered.html.contains("<h1 id=\"section-1\">"));
        assert!(rendered.html.contains("<h2 id=\"section-2\">"));
        assert!(rendered.html.contains("<h4 id=\"section-3\">"));
        assert!(rendered.html.contains("href=\"https://example.com\""));
        assert!(
            !rendered
                .headings
                .iter()
                .any(|heading| heading.text == "not a heading")
        );
    }

    #[test]
    fn markdown_empty_heading_has_an_id_but_no_outline_entry() {
        let rendered = render_markdown("#\n\nParagraph");

        assert!(rendered.headings.is_empty());
        assert!(rendered.html.contains("<h1 id=\"section-1\"></h1>"));
    }
}
