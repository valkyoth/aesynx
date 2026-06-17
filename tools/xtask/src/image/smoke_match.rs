use std::fs;
use std::path::Path;

use super::SmokeKind;

pub fn serial_log_contains_marker(path: &Path, smoke: SmokeKind) -> bool {
    fs::read_to_string(path).is_ok_and(|contents| serial_log_contents_match(&contents, smoke))
}

pub(crate) fn serial_log_contents_match(contents: &str, smoke: SmokeKind) -> bool {
    contains_all(contents, smoke.required_markers())
        && contains_none(contents, smoke.forbidden_markers())
}

fn contains_all(contents: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .all(|marker| contains_marker(contents, marker))
}

fn contains_none(contents: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .all(|marker| !contains_marker(contents, marker))
}

fn contains_marker(contents: &str, marker: &str) -> bool {
    if !marker.contains('=') {
        return contents.contains(marker);
    }

    let mut offset = 0usize;
    while let Some(relative_start) = contents[offset..].find(marker) {
        let start = offset + relative_start;
        let end = start + marker.len();
        if is_marker_boundary(contents[..start].chars().next_back())
            && (is_value_prefix_marker(marker)
                || is_marker_boundary(contents[end..].chars().next()))
        {
            return true;
        }
        offset = end;
    }
    false
}

fn is_value_prefix_marker(marker: &str) -> bool {
    marker.ends_with('=') || marker.ends_with("=0x")
}

fn is_marker_boundary(character: Option<char>) -> bool {
    match character {
        None => true,
        Some(character) => character.is_ascii_whitespace() || character == ',',
    }
}
