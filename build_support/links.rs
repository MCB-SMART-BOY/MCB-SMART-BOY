use std::{error::Error, fmt};

#[derive(Debug)]
pub(crate) enum ContentLinkError {
    /// Queries, percent encoding, or backslashes make a content path ambiguous.
    UnsupportedPath,
    /// The source document is not rooted under the writing namespace.
    InvalidSource,
    /// A relative traversal would leave the content root.
    EscapesRoot,
    /// A relative path contains an empty component.
    EmptyComponent,
    /// A Markdown target has no valid article filename.
    InvalidArticle,
}

impl fmt::Display for ContentLinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPath => "unsupported Markdown content path",
            Self::InvalidSource => "invalid source page path",
            Self::EscapesRoot => "Markdown link escapes content root",
            Self::EmptyComponent => "empty Markdown link component",
            Self::InvalidArticle => "invalid Markdown article filename",
        })
    }
}

impl Error for ContentLinkError {}

/// Resolve a relative Markdown content link to its canonical page path.
/// Non-content destinations are handled by the Markdown renderer's existing safety policy.
pub(crate) fn resolve_content_link(
    source_path: &str,
    destination: &str,
) -> Result<Option<String>, ContentLinkError> {
    let (relative, fragment) = destination.split_once('#').unwrap_or((destination, ""));
    if relative.starts_with('/') || relative.contains(':') {
        return Ok(None);
    }
    if relative.contains(".md?")
        || (relative.ends_with(".md") && relative.contains(&['?', '%', '\\'][..]))
    {
        return Err(ContentLinkError::UnsupportedPath);
    }
    if !relative.ends_with(".md") {
        return Ok(None);
    }
    let parent = source_path
        .rsplit_once('/')
        .map_or("", |(parent, _)| parent);
    let mut segments: Vec<&str> = parent.trim_start_matches('/').split('/').collect();
    if segments.first() != Some(&"writing") {
        return Err(ContentLinkError::InvalidSource);
    }
    for component in relative.split('/') {
        match component {
            "." => {}
            ".." if segments.len() > 1 => {
                segments.pop();
            }
            ".." => return Err(ContentLinkError::EscapesRoot),
            "" => return Err(ContentLinkError::EmptyComponent),
            part => segments.push(part),
        }
    }
    let Some(last) = segments.pop() else {
        return Err(ContentLinkError::InvalidArticle);
    };
    if last != "_index.md" {
        let Some(stem) = last.strip_suffix(".md") else {
            return Err(ContentLinkError::InvalidArticle);
        };
        if stem.is_empty() {
            return Err(ContentLinkError::InvalidArticle);
        }
        segments.push(stem);
    }
    // URLs are directory-shaped even though Markdown sources remain file-shaped.
    let mut path = format!("/{}/", segments.join("/"));
    if !fragment.is_empty() {
        path.push('#');
        path.push_str(fragment);
    }
    Ok(Some(path))
}
