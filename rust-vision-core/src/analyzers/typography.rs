//! Typography analyzer — clusters elements by font characteristics and
//! flags inconsistencies. e.g., "16 text elements but 4 distinct
//! font-family declarations" usually indicates an unsynced design system.

use std::collections::BTreeMap;

use super::{Finding, Severity};
use crate::element_snapshot::ElementSnapshot;

pub fn run(snapshot: &ElementSnapshot) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Cluster by (rounded font_size, font_family). Skip elements without
    // typography data.
    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut size_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut text_elements = 0;
    for el in &snapshot.elements {
        if el.text.is_none() || el.text.as_deref() == Some("") {
            continue;
        }
        text_elements += 1;
        if let Some(fam) = &el.font_family {
            *family_counts.entry(fam.clone()).or_default() += 1;
        }
        if let Some(size) = el.font_size_px {
            // Bucket by 2-px granularity; tiny differences are not visually
            // distinct and shouldn't count as separate sizes.
            let bucket = ((size / 2.0).round() as u32) * 2;
            *size_counts.entry(bucket).or_default() += 1;
        }
    }

    if text_elements == 0 {
        return findings;
    }

    // Heuristic: more than 3 distinct font families across text elements
    // is almost always a design-system drift.
    if family_counts.len() > 3 {
        findings.push(Finding::new(
            "font_family_drift",
            Severity::Warning,
            format!(
                "{} distinct font-family declarations across {} text elements",
                family_counts.len(),
                text_elements
            ),
        ));
    }

    // More than 8 distinct font sizes is a similar smell — design systems
    // typically use 4-6 sizes (h1..h4, body, caption).
    if size_counts.len() > 8 {
        findings.push(Finding::new(
            "font_size_drift",
            Severity::Info,
            format!(
                "{} distinct font sizes across {} text elements",
                size_counts.len(),
                text_elements
            ),
        ));
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_snapshot::Element;
    use crate::frame::Region;

    fn text_el(id: &str, family: &str, size: f32) -> Element {
        Element {
            id: id.to_string(),
            bbox: Region {
                x: 0,
                y: 0,
                w: 10,
                h: 10,
            },
            text: Some("x".into()),
            role: None,
            interactable: false,
            fg_color: None,
            bg_color: None,
            font_size_px: Some(size),
            font_family: Some(family.into()),
            line_height_px: None,
            parent_id: None,
            children_ids: vec![],
        }
    }

    #[test]
    fn flags_many_distinct_families() {
        let snap = ElementSnapshot {
            elements: vec![
                text_el("a", "Inter", 16.0),
                text_el("b", "Roboto", 16.0),
                text_el("c", "Arial", 16.0),
                text_el("d", "Helvetica", 16.0),
            ],
        };
        let findings = run(&snap);
        assert!(findings.iter().any(|f| f.kind == "font_family_drift"));
    }

    #[test]
    fn no_findings_when_consistent() {
        let snap = ElementSnapshot {
            elements: vec![
                text_el("a", "Inter", 14.0),
                text_el("b", "Inter", 16.0),
                text_el("c", "Inter", 24.0),
            ],
        };
        let findings = run(&snap);
        assert!(findings.is_empty());
    }
}
