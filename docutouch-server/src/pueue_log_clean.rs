use std::panic::{AssertUnwindSafe, catch_unwind};

use unicode_width::UnicodeWidthStr as _;

const SCREEN_ROWS: u16 = 256;
const SCREEN_COLS: u16 = 512;
const SCROLLBACK_PADDING_ROWS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SegmentKind {
    Main,
    AlternateScreen,
}

#[derive(Debug)]
struct Segment {
    kind: SegmentKind,
    raw: String,
}

#[derive(Debug)]
struct HyperlinkState {
    url: String,
    label_anchor: usize,
}

pub(crate) fn clean_task_log_surface(raw: &str) -> String {
    catch_unwind(AssertUnwindSafe(|| clean_task_log_surface_inner(raw)))
        .unwrap_or_else(|_| fallback_projection(raw))
}

fn clean_task_log_surface_inner(raw: &str) -> String {
    let mut cleaned = String::new();
    for segment in scan_segments(raw) {
        cleaned.push_str(&project_segment(&segment));
    }
    cleaned
}

fn scan_segments(raw: &str) -> Vec<Segment> {
    let bytes = raw.as_bytes();
    let mut segments = Vec::new();
    let mut current_kind = SegmentKind::Main;
    let mut current = String::new();
    let mut hyperlink = None;
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == 0x1b {
            if let Some((payload, consumed)) = parse_osc_payload(raw, index) {
                handle_osc_payload(payload, &mut current, &mut hyperlink);
                index += consumed;
                continue;
            }

            if let Some((switch, consumed)) = parse_alt_screen_switch(bytes, index) {
                close_hyperlink(&mut current, &mut hyperlink);
                flush_segment(&mut segments, current_kind, &mut current);
                current_kind = switch;
                index += consumed;
                continue;
            }
        }

        let ch = raw[index..]
            .chars()
            .next()
            .expect("valid utf-8 boundary while scanning task log");
        current.push(ch);
        index += ch.len_utf8();
    }

    close_hyperlink(&mut current, &mut hyperlink);
    flush_segment(&mut segments, current_kind, &mut current);
    segments
}

fn flush_segment(segments: &mut Vec<Segment>, kind: SegmentKind, current: &mut String) {
    if current.is_empty() {
        return;
    }

    segments.push(Segment {
        kind,
        raw: std::mem::take(current),
    });
}

fn parse_osc_payload(raw: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = raw.as_bytes();
    if bytes.get(start + 1) != Some(&b']') {
        return None;
    }

    let payload_start = start + 2;
    let mut index = payload_start;
    while index < bytes.len() {
        match bytes[index] {
            0x07 => return Some((&raw[payload_start..index], index + 1 - start)),
            0x1b if bytes.get(index + 1) == Some(&b'\\') => {
                return Some((&raw[payload_start..index], index + 2 - start));
            }
            b'\n' | b'\r' => return Some((&raw[payload_start..index], index - start)),
            _ => index += 1,
        }
    }

    Some((&raw[payload_start..], raw.len() - start))
}

fn handle_osc_payload(payload: &str, current: &mut String, hyperlink: &mut Option<HyperlinkState>) {
    let mut parts = payload.splitn(3, ';');
    let command = parts.next().unwrap_or_default();
    if command != "8" {
        return;
    }

    let _params = parts.next().unwrap_or_default();
    let url = parts.next().unwrap_or_default();
    if url.is_empty() {
        close_hyperlink(current, hyperlink);
        return;
    }

    close_hyperlink(current, hyperlink);
    *hyperlink = Some(HyperlinkState {
        url: url.to_string(),
        label_anchor: current.len(),
    });
}

fn close_hyperlink(current: &mut String, hyperlink: &mut Option<HyperlinkState>) {
    let Some(HyperlinkState { url, label_anchor }) = hyperlink.take() else {
        return;
    };
    if current.len() == label_anchor {
        return;
    }

    current.push_str(" (");
    current.push_str(&url);
    current.push(')');
}

fn parse_alt_screen_switch(bytes: &[u8], start: usize) -> Option<(SegmentKind, usize)> {
    if bytes.get(start + 1) != Some(&b'[') || bytes.get(start + 2) != Some(&b'?') {
        return None;
    }

    let mut index = start + 3;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_digit() || byte == b';' {
            index += 1;
            continue;
        }

        if byte != b'h' && byte != b'l' {
            return None;
        }

        let params = std::str::from_utf8(&bytes[start + 3..index]).ok()?;
        if !params
            .split(';')
            .any(|param| matches!(param, "47" | "1047" | "1049"))
        {
            return None;
        }

        let next_kind = if byte == b'h' {
            SegmentKind::AlternateScreen
        } else {
            SegmentKind::Main
        };
        return Some((next_kind, index + 1 - start));
    }

    None
}

fn project_segment(segment: &Segment) -> String {
    if segment.raw.is_empty() {
        return String::new();
    }

    if segment.kind == SegmentKind::AlternateScreen || segment_requires_screen_path(&segment.raw) {
        return screen_projection(&rewrite_screen_controls(&segment.raw));
    }

    plain_projection(&segment.raw)
}

fn segment_requires_screen_path(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            0x1b => return true,
            b'\r' if bytes.get(index + 1) != Some(&b'\n') => return true,
            0x00..=0x08 | 0x0b..=0x1f | 0x7f => return true,
            _ => {}
        }

        let ch = raw[index..]
            .chars()
            .next()
            .expect("valid utf-8 boundary while classifying task log segment");
        index += ch.len_utf8();
    }

    false
}

fn rewrite_screen_controls(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut rewritten = String::with_capacity(raw.len());
    let mut index = 0usize;
    let mut previous_was_cr = false;

    while index < bytes.len() {
        let ch = raw[index..]
            .chars()
            .next()
            .expect("valid utf-8 boundary while rewriting task log controls");
        match ch {
            '\r' => {
                if bytes.get(index + 1) == Some(&b'\n')
                    || next_is_line_clear_escape(bytes, index + 1)
                {
                    rewritten.push('\r');
                } else {
                    // Treat bare carriage-return redraws as frame replacement, not literal overprint history.
                    rewritten.push_str("\x1b[2K\r");
                }
                previous_was_cr = true;
            }
            '\n' => {
                if !previous_was_cr {
                    rewritten.push('\r');
                }
                rewritten.push('\n');
                previous_was_cr = false;
            }
            _ => {
                rewritten.push(ch);
                previous_was_cr = false;
            }
        }

        index += ch.len_utf8();
    }

    rewritten
}

fn next_is_line_clear_escape(bytes: &[u8], start: usize) -> bool {
    matches!(
        bytes.get(start..),
        Some([0x1b, b'[', b'K', ..])
            | Some([0x1b, b'[', b'0', b'K', ..])
            | Some([0x1b, b'[', b'1', b'K', ..])
            | Some([0x1b, b'[', b'2', b'K', ..])
    )
}

fn plain_projection(raw: &str) -> String {
    let mut plain = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    plain.push('\n');
                }
            }
            '\u{0000}'..='\u{0008}' | '\u{000B}'..='\u{001F}' | '\u{007F}' => {}
            _ => plain.push(ch),
        }
    }
    plain
}

fn screen_projection(raw: &str) -> String {
    catch_unwind(AssertUnwindSafe(|| {
        let mut parser = vt100::Parser::new(SCREEN_ROWS, SCREEN_COLS, estimate_scrollback_len(raw));
        parser.process(raw.as_bytes());
        let mut rendered = parser.screen().contents();
        if ends_with_line_break(raw) && !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        rendered
    }))
    .unwrap_or_else(|_| fallback_projection(raw))
}

fn ends_with_line_break(raw: &str) -> bool {
    raw.ends_with('\n') || raw.ends_with('\r')
}

fn estimate_scrollback_len(raw: &str) -> usize {
    let cols = usize::from(SCREEN_COLS);
    let mut rows = usize::from(SCREEN_ROWS) + SCROLLBACK_PADDING_ROWS;
    for line in raw.split('\n') {
        let width = line.width();
        rows = rows.saturating_add((width / cols).saturating_add(1));
    }
    rows
}

fn fallback_projection(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut cleaned = String::with_capacity(raw.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == 0x1b {
            if let Some((_, consumed)) = parse_osc_payload(raw, index) {
                index += consumed;
                continue;
            }
            if let Some((_, consumed)) = parse_alt_screen_switch(bytes, index) {
                index += consumed;
                continue;
            }
            if let Some(consumed) = consume_escape_sequence(bytes, index) {
                index += consumed;
                continue;
            }
            index += 1;
            continue;
        }

        let ch = raw[index..]
            .chars()
            .next()
            .expect("valid utf-8 boundary while building fallback task log projection");
        match ch {
            '\r' => cleaned.push('\n'),
            '\u{0000}'..='\u{0008}' | '\u{000B}'..='\u{001F}' | '\u{007F}' => {}
            _ => cleaned.push(ch),
        }
        index += ch.len_utf8();
    }
    cleaned
}

fn consume_escape_sequence(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes.get(start + 1)? {
        b'[' => {
            let mut index = start + 2;
            while index < bytes.len() {
                if (0x40..=0x7e).contains(&bytes[index]) {
                    return Some(index + 1 - start);
                }
                index += 1;
            }
            Some(bytes.len() - start)
        }
        _ => Some(2),
    }
}
