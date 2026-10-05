use serde::Deserialize;
use std::{error::Error, fmt, fs, io, path::Path};

const MAX_SOURCE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_METADATA_BYTES: usize = 16 * 1024;
const LEAP_YEAR_INTERVAL: u32 = 4;
const CENTURY_YEARS: u32 = 100;
const GREGORIAN_CYCLE_YEARS: u32 = 400;

#[derive(Debug)]
pub(crate) enum ContentError {
    /// A content-tree filesystem operation failed; `source` retains the OS error.
    Io {
        path: String,
        operation: &'static str,
        source: io::Error,
    },
    /// The bounded YAML parser rejected a document; `source` retains its diagnostics.
    /// Box large diagnostics so successful content loading carries a small error variant.
    Yaml {
        path: String,
        source: Box<serde_saphyr::Error>,
    },
    /// A user-fixable Markdown target failed validation; retain the resolver error.
    Link {
        path: String,
        destination: String,
        source: super::links::ContentLinkError,
    },
    /// A user-fixable content or metadata constraint failed.
    Invalid {
        path: String,
        field: &'static str,
        reason: String,
    },
}
impl fmt::Display for ContentError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                path,
                operation,
                source,
            } => write!(fmt, "{operation} {path}: {source}"),
            Self::Yaml { path, source } => {
                write!(fmt, "parse YAML front matter at {path}: {source}")
            }
            Self::Link {
                path,
                destination,
                source,
            } => write!(
                fmt,
                "resolve Markdown link {destination} at {path}: {source}"
            ),
            Self::Invalid {
                path,
                field,
                reason,
            } => write!(fmt, "invalid {field} at {path}: {reason}"),
        }
    }
}
impl Error for ContentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Yaml { source, .. } => Some(source.as_ref()),
            Self::Link { source, .. } => Some(source),
            Self::Invalid { .. } => None,
        }
    }
}
pub(crate) fn invalid(path: &Path, field: &'static str, reason: impl Into<String>) -> ContentError {
    ContentError::Invalid {
        path: path.display().to_string(),
        field,
        reason: reason.into(),
    }
}
pub(crate) fn io_error(path: &Path, operation: &'static str, source: io::Error) -> ContentError {
    ContentError::Io {
        path: path.display().to_string(),
        operation,
        source,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectoryMeta {
    pub(crate) title: String,
    pub(crate) weight: Option<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArticleMeta {
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) date: String,
    pub(crate) category: Option<String>,
    pub(crate) reading_minutes: u8,
    pub(crate) weight: Option<u32>,
}

pub(crate) fn read_content(path: &Path) -> Result<(String, String), ContentError> {
    let metadata = fs::symlink_metadata(path).map_err(|err| io_error(path, "inspect", err))?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(invalid(
            path,
            "file",
            "expected an ordinary file no larger than 2 MiB (no symbolic links)",
        ));
    }
    let source = fs::read_to_string(path).map_err(|err| io_error(path, "read UTF-8", err))?;
    let Some(rest) = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
    else {
        return Err(invalid(path, "front matter", "must start with a --- line"));
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(&['\r', '\n'][..]) == "---" {
            if offset > MAX_METADATA_BYTES {
                return Err(invalid(path, "front matter", "exceeds 16 KiB"));
            }
            return Ok((
                rest[..offset].to_owned(),
                rest[offset + line.len()..].to_owned(),
            ));
        }
        offset += line.len();
        if offset > MAX_METADATA_BYTES {
            return Err(invalid(path, "front matter", "exceeds 16 KiB"));
        }
    }
    Err(invalid(path, "front matter", "missing closing --- line"))
}

pub(crate) fn parse_metadata<T: for<'de> Deserialize<'de>>(
    path: &Path,
    yaml: &str,
) -> Result<T, ContentError> {
    let options = serde_saphyr::options! {
        reject_unsupported_tags: true,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error,
        budget: serde_saphyr::budget! {
            max_documents: 1,
            max_anchors: 0,
            max_aliases: 0,
            max_depth: 4,
            max_events: 256,
            max_nodes: 128,
            max_total_scalar_bytes: MAX_METADATA_BYTES,
        },
    };
    let documents = serde_saphyr::from_multiple_with_options(yaml, options).map_err(|source| {
        ContentError::Yaml {
            path: path.display().to_string(),
            source: Box::new(source),
        }
    })?;
    let [metadata]: [T; 1] = documents
        .try_into()
        .map_err(|_| invalid(path, "front matter", "expected exactly one YAML document"))?;
    Ok(metadata)
}

pub(crate) fn require_text(
    path: &Path,
    field: &'static str,
    text: &str,
) -> Result<(), ContentError> {
    if text.trim().is_empty() {
        return Err(invalid(path, field, "must not be blank"));
    }
    Ok(())
}

pub(crate) fn validate_date(path: &Path, date: &str) -> Result<String, ContentError> {
    let bytes = date.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, c)| i != 4 && i != 7 && !c.is_ascii_digit())
    {
        return Err(invalid(path, "date", "expected YYYY-MM-DD"));
    }
    let year: u32 = date[..4]
        .parse()
        .map_err(|err| invalid(path, "date", format!("invalid year: {err}")))?;
    let month: u32 = date[5..7]
        .parse()
        .map_err(|err| invalid(path, "date", format!("invalid month: {err}")))?;
    let day: u32 = date[8..]
        .parse()
        .map_err(|err| invalid(path, "date", format!("invalid day: {err}")))?;
    let is_leap = year.is_multiple_of(LEAP_YEAR_INTERVAL)
        && (!year.is_multiple_of(CENTURY_YEARS) || year.is_multiple_of(GREGORIAN_CYCLE_YEARS));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > days {
        return Err(invalid(
            path,
            "date",
            format!("not a calendar date: {date}"),
        ));
    }
    Ok(format!("{}.{} / {}", &date[5..7], &date[8..], &date[..4]))
}
