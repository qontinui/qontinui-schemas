//! Post-encode metadata stripping for PNG / JPEG / WebP byte streams.
//!
//! Each format has a well-defined container structure with a small set of
//! "essential" chunks/markers; everything else is metadata that must be
//! removed under [`MetadataPolicy::StripAll`](crate::MetadataPolicy::StripAll).

use crate::error::VisionError;

/// Strip ancillary chunks from a PNG byte stream. Keeps IHDR, PLTE, IDAT, IEND,
/// and tRNS (transparency, structurally needed for indexed/grayscale images
/// that carry an alpha key). Drops gAMA, cHRM, iCCP, tEXt, iTXt, zTXt, sBIT,
/// sRGB, bKGD, hIST, pHYs, sPLT, tIME, and any other ancillary chunks.
pub fn strip_png(bytes: &[u8]) -> Result<Vec<u8>, VisionError> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 8 || &bytes[..8] != SIGNATURE {
        return Err(VisionError::DecodeFailed(
            "not a PNG (bad signature)".into(),
        ));
    }

    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(SIGNATURE);

    let mut i = 8usize;
    while i + 12 <= bytes.len() {
        let length =
            u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
        let chunk_type = &bytes[i + 4..i + 8];
        let total = 12usize
            .checked_add(length)
            .ok_or_else(|| VisionError::DecodeFailed("PNG chunk length overflow".into()))?;
        if i + total > bytes.len() {
            return Err(VisionError::DecodeFailed(
                "PNG chunk runs past end of buffer".into(),
            ));
        }
        let keep = matches!(chunk_type, b"IHDR" | b"PLTE" | b"IDAT" | b"IEND" | b"tRNS");
        if keep {
            out.extend_from_slice(&bytes[i..i + total]);
        }
        if chunk_type == b"IEND" {
            return Ok(out);
        }
        i += total;
    }
    Err(VisionError::DecodeFailed("PNG missing IEND".into()))
}

/// Strip APPn (0xE0..=0xEF) and COM (0xFE) markers from a JPEG byte stream.
/// Keeps SOI/EOI, SOF, SOS, DQT, DHT, DAC, DNL, DRI, DHP, EXP, JPG, and RST markers.
/// SOS is special: stand-alone marker followed by an entropy-coded segment which
/// is *not* length-prefixed; we copy bytes verbatim until the next marker.
pub fn strip_jpeg(bytes: &[u8]) -> Result<Vec<u8>, VisionError> {
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(VisionError::DecodeFailed("not a JPEG (bad SOI)".into()));
    }
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(&bytes[..2]); // SOI

    let mut i = 2usize;
    while i + 1 < bytes.len() {
        // Skip fill bytes (0xFF padding).
        while i < bytes.len() && bytes[i] != 0xFF {
            i += 1;
        }
        while i + 1 < bytes.len() && bytes[i] == 0xFF && bytes[i + 1] == 0xFF {
            i += 1;
        }
        if i + 1 >= bytes.len() {
            break;
        }
        let marker = bytes[i + 1];

        // EOI: copy and finish.
        if marker == 0xD9 {
            out.push(0xFF);
            out.push(0xD9);
            return Ok(out);
        }

        // RSTn (0xD0..=0xD7) and TEM (0x01) are standalone (no length, no payload).
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 || marker == 0x00 {
            out.push(0xFF);
            out.push(marker);
            i += 2;
            continue;
        }

        // SOS: copy marker + length-prefixed segment, then copy the entropy-coded
        // data until we hit the next non-RST marker.
        if marker == 0xDA {
            if i + 4 > bytes.len() {
                return Err(VisionError::DecodeFailed("JPEG SOS truncated".into()));
            }
            let seg_len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            let segment_end = i + 2 + seg_len;
            if segment_end > bytes.len() {
                return Err(VisionError::DecodeFailed("JPEG SOS length overflow".into()));
            }
            out.extend_from_slice(&bytes[i..segment_end]);
            // Copy entropy-coded data verbatim until next non-RST marker.
            i = segment_end;
            while i + 1 < bytes.len() {
                if bytes[i] == 0xFF {
                    let next = bytes[i + 1];
                    // 0xFF00 is a stuffed byte (literal 0xFF in data).
                    // RSTn markers are in-band; keep them.
                    if next == 0x00 || (0xD0..=0xD7).contains(&next) {
                        out.push(bytes[i]);
                        out.push(next);
                        i += 2;
                        continue;
                    }
                    // Any other marker terminates the entropy section.
                    break;
                }
                out.push(bytes[i]);
                i += 1;
            }
            continue;
        }

        // APPn (0xE0..=0xEF) and COM (0xFE) are metadata — drop them.
        let drop = (0xE0..=0xEF).contains(&marker) || marker == 0xFE;

        if i + 4 > bytes.len() {
            return Err(VisionError::DecodeFailed("JPEG segment truncated".into()));
        }
        let seg_len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if seg_len < 2 {
            return Err(VisionError::DecodeFailed("JPEG segment length < 2".into()));
        }
        let segment_end = i + 2 + seg_len;
        if segment_end > bytes.len() {
            return Err(VisionError::DecodeFailed(
                "JPEG segment length overflow".into(),
            ));
        }
        if !drop {
            out.extend_from_slice(&bytes[i..segment_end]);
        }
        i = segment_end;
    }
    Err(VisionError::DecodeFailed("JPEG missing EOI".into()))
}

/// Strip metadata chunks (EXIF, XMP, ICCP) from a WebP byte stream. Keeps
/// VP8/VP8L/VP8X and ANIM/ANMF/ALPH chunks (animation + alpha are structurally
/// part of the bitstream). The container is a RIFF: 12-byte header followed by
/// 8-byte-headered chunks (chunk-id + size, big-endian neither — RIFF is LE).
pub fn strip_webp(bytes: &[u8]) -> Result<Vec<u8>, VisionError> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return Err(VisionError::DecodeFailed(
            "not a WebP (bad RIFF/WEBP)".into(),
        ));
    }
    // Body starts with the "WEBP" form-type marker; followed by the kept chunks.
    let mut body = Vec::with_capacity(bytes.len() - 8);
    body.extend_from_slice(b"WEBP");

    let mut i = 12usize;
    while i + 8 <= bytes.len() {
        let chunk_id = &bytes[i..i + 4];
        let size =
            u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]) as usize;
        // Chunks are padded to even length.
        let padded = size + (size & 1);
        let total = 8usize
            .checked_add(padded)
            .ok_or_else(|| VisionError::DecodeFailed("WebP chunk length overflow".into()))?;
        if i + total > bytes.len() {
            return Err(VisionError::DecodeFailed(
                "WebP chunk runs past end of buffer".into(),
            ));
        }
        let keep = matches!(
            chunk_id,
            b"VP8 " | b"VP8L" | b"VP8X" | b"ANIM" | b"ANMF" | b"ALPH"
        );
        if keep {
            body.extend_from_slice(&bytes[i..i + total]);
        }
        i += total;
    }

    // RIFF size is the byte count *after* the 8-byte RIFF header — i.e. the body
    // length (which already starts with the WEBP form-type marker).
    let new_size = (body.len() as u32).to_le_bytes();
    let mut out = Vec::with_capacity(8 + body.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&new_size);
    out.extend_from_slice(&body);
    Ok(out)
}

/// Re-parse stripped bytes and verify no disallowed-for-`StripAll` items remain.
/// Used by [`Stage::Verify`](crate::Stage::Verify) when the contract demands
/// [`MetadataPolicy::StripAll`](crate::MetadataPolicy::StripAll).
pub fn assert_stripped_png(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("not a PNG".into());
    }
    let mut i = 8usize;
    while i + 12 <= bytes.len() {
        let length =
            u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
        let chunk_type = &bytes[i + 4..i + 8];
        let total = 12 + length;
        if i + total > bytes.len() {
            return Err("truncated chunk".into());
        }
        let allowed = matches!(chunk_type, b"IHDR" | b"PLTE" | b"IDAT" | b"IEND" | b"tRNS");
        if !allowed {
            return Err(format!(
                "disallowed PNG chunk: {}",
                String::from_utf8_lossy(chunk_type)
            ));
        }
        if chunk_type == b"IEND" {
            return Ok(());
        }
        i += total;
    }
    Err("missing IEND".into())
}

pub fn assert_stripped_jpeg(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err("not a JPEG".into());
    }
    let mut i = 2usize;
    while i + 1 < bytes.len() {
        while i < bytes.len() && bytes[i] != 0xFF {
            i += 1;
        }
        while i + 1 < bytes.len() && bytes[i] == 0xFF && bytes[i + 1] == 0xFF {
            i += 1;
        }
        if i + 1 >= bytes.len() {
            return Err("ran off end before EOI".into());
        }
        let marker = bytes[i + 1];
        if marker == 0xD9 {
            return Ok(());
        }
        if (0xE0..=0xEF).contains(&marker) {
            return Err(format!("disallowed JPEG APP{:X} marker", marker - 0xE0));
        }
        if marker == 0xFE {
            return Err("disallowed JPEG COM marker".into());
        }
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 || marker == 0x00 {
            i += 2;
            continue;
        }
        if marker == 0xDA {
            // SOS — skip segment then scan entropy data for next marker.
            if i + 4 > bytes.len() {
                return Err("SOS truncated".into());
            }
            let seg_len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            i = i + 2 + seg_len;
            while i + 1 < bytes.len() {
                if bytes[i] == 0xFF {
                    let next = bytes[i + 1];
                    if next == 0x00 || (0xD0..=0xD7).contains(&next) {
                        i += 2;
                        continue;
                    }
                    break;
                }
                i += 1;
            }
            continue;
        }
        if i + 4 > bytes.len() {
            return Err("segment truncated".into());
        }
        let seg_len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        i += 2 + seg_len;
    }
    Err("missing EOI".into())
}

pub fn assert_stripped_webp(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return Err("not a WebP".into());
    }
    let mut i = 12usize;
    while i + 8 <= bytes.len() {
        let chunk_id = &bytes[i..i + 4];
        let size =
            u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]) as usize;
        let padded = size + (size & 1);
        let total = 8 + padded;
        if i + total > bytes.len() {
            return Err("truncated WebP chunk".into());
        }
        let allowed = matches!(
            chunk_id,
            b"VP8 " | b"VP8L" | b"VP8X" | b"ANIM" | b"ANMF" | b"ALPH"
        );
        if !allowed {
            return Err(format!(
                "disallowed WebP chunk: {}",
                String::from_utf8_lossy(chunk_id)
            ));
        }
        i += total;
    }
    Ok(())
}
