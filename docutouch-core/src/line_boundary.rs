pub(crate) fn normalize_payload_for_target_boundaries(
    payload: Vec<u8>,
    target_bytes: &[u8],
    needs_prefix: bool,
    needs_suffix: bool,
) -> Vec<u8> {
    if payload.is_empty() || (!needs_prefix && !needs_suffix) {
        return payload;
    }

    let newline = preferred_target_newline(target_bytes);
    let mut normalized = Vec::with_capacity(
        payload.len() + newline.len() * (usize::from(needs_prefix) + usize::from(needs_suffix)),
    );
    if needs_prefix {
        normalized.extend_from_slice(newline);
    }
    normalized.extend_from_slice(&payload);
    if needs_suffix {
        normalized.extend_from_slice(newline);
    }
    normalized
}

pub(crate) fn normalize_replacement_payload_for_result_suffix(
    payload: Vec<u8>,
    target_bytes: &[u8],
    suffix_boundary: usize,
) -> Vec<u8> {
    if payload.is_empty()
        || ends_with_line_break(&payload)
        || !needs_boundary_suffix(target_bytes, suffix_boundary)
    {
        return payload;
    }

    let newline = preferred_target_newline(target_bytes);
    let mut normalized = Vec::with_capacity(payload.len() + newline.len());
    normalized.extend_from_slice(&payload);
    normalized.extend_from_slice(newline);
    normalized
}

pub(crate) fn needs_boundary_prefix(target_bytes: &[u8], boundary: usize) -> bool {
    boundary > 0 && !is_line_break_byte(target_bytes[boundary - 1])
}

pub(crate) fn needs_boundary_suffix(target_bytes: &[u8], boundary: usize) -> bool {
    boundary < target_bytes.len() && !is_line_break_byte(target_bytes[boundary])
}

pub(crate) fn preferred_target_newline(target_bytes: &[u8]) -> &'static [u8] {
    if target_bytes.windows(2).any(|window| window == b"\r\n") {
        b"\r\n"
    } else {
        b"\n"
    }
}

pub(crate) fn ends_with_line_break(bytes: &[u8]) -> bool {
    matches!(bytes.last(), Some(b'\n') | Some(b'\r'))
}

pub(crate) fn is_line_break_byte(byte: u8) -> bool {
    byte == b'\n' || byte == b'\r'
}
