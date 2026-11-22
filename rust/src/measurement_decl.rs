//! The grammar of `.qontinui/measurements.toml`, and its ONE parser.
//!
//! Plan `2026-11-22-phase1-envelope-frozen-and-measurement-dispatch-proven`
//! (D3, Phase 4). A measured repository declares the measurements coord may
//! dispatch against it in a dedicated file at its default-branch tip:
//!
//! ```toml
//! [[measurements]]
//! name = "unit-suite"
//! source = "cargo-test"
//! subject = "qontinui-types"
//! task = "unit"
//! ```
//!
//! Coord reads the file from its canonical mirror, and a measurement dispatch
//! writes one `coord.run_outcomes` row labelled with the declaration's
//! `source` / `subject` / `task` — so the labels come from the repo's own
//! declaration, never from a request body.
//!
//! The parser is STRICT: the file is dedicated, so an unknown key, at the top
//! level or on an entry, is refused rather than ignored (a typo would
//! otherwise silently drop a field). Version 1 carries NO `job` key: the
//! runner's dispatch payload runs the whole `steps` list, so a named job
//! would label a run of something else. A `job` key is refused by name
//! ([`MeasurementDeclError::JobKeyNotSupported`]); job selection arrives as
//! grammar version 2.
//!
//! Labels follow the `run_outcomes` rules
//! ([`crate::run_outcomes::is_valid_run_outcome_label`]), so every
//! declaration this parser accepts is a row coord's result door can write.
//! A measurement `name` is a slug — the metric-name grammar
//! ([`crate::run_outcomes::is_valid_metric_name`], `[A-Za-z0-9_.:-]`), and not
//! made only of dots — since it is a dispatch key, part of a UUIDv5 input and
//! a URL/MCP argument, where `.` or `..` would read as a path segment.
//!
//! [`parse_measurements`] is the ONLY entry point for repository text. The
//! types derive `Deserialize` so a stored snapshot can be read back, but
//! deserializing them directly validates nothing beyond shape.
//!
//! Behind the `measurement-decl` feature, which is what pulls in `toml`.
//!
//! PURE: no I/O, no clock, no coord state.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::run_outcomes::{
    is_valid_metric_name, is_valid_run_outcome_label, RUN_OUTCOME_LABEL_MAX_BYTES,
    RUN_OUTCOME_METRIC_NAME_MAX_BYTES,
};

/// Where the declaration lives, relative to the measured repository's root.
pub const MEASUREMENTS_FILE_PATH: &str = ".qontinui/measurements.toml";

/// Byte bound on the file, checked before parsing.
pub const MEASUREMENTS_FILE_MAX_BYTES: usize = 64 * 1024;

/// Most measurements one file may declare.
pub const MEASUREMENTS_MAX_ENTRIES: usize = 64;

/// The key version 1 refuses by name (see the module docs).
pub const JOB_KEY: &str = "job";

/// The whole file: `[[measurements]]` entries and nothing else.
///
/// `Deserialize` checks shape only (strict keys, string values). It does NOT
/// apply the label, name, duplicate or size rules — repository text goes
/// through [`parse_measurements`], never through a bare deserialize.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementsFile {
    /// The declared measurements, in file order. Absent means none.
    #[serde(default)]
    pub measurements: Vec<MeasurementDecl>,
}

/// One declared measurement. Serializes to the snapshot coord stores on the
/// dispatch row (`coord.ci_dispatches.measurement`).
///
/// `Deserialize` checks shape only; it does NOT validate the labels or the
/// name. Repository text goes through [`parse_measurements`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementDecl {
    /// The name a caller dispatches by; unique within the file.
    pub name: String,
    /// The `run_outcomes` `source` label of the row a run writes.
    pub source: String,
    /// The `run_outcomes` `subject` label.
    pub subject: String,
    /// The `run_outcomes` `task` label.
    pub task: String,
}

impl MeasurementsFile {
    /// The declaration named `name`, if the file declares it.
    pub fn get(&self, name: &str) -> Option<&MeasurementDecl> {
        self.measurements.iter().find(|m| m.name == name)
    }
}

/// Which label field of an entry failed the label rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementField {
    Source,
    Subject,
    Task,
}

impl MeasurementField {
    pub fn as_str(self) -> &'static str {
        match self {
            MeasurementField::Source => "source",
            MeasurementField::Subject => "subject",
            MeasurementField::Task => "task",
        }
    }
}

/// Why a file was refused. `index` is the zero-based `[[measurements]]` entry.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MeasurementDeclError {
    #[error(
        "{MEASUREMENTS_FILE_PATH} is {bytes} bytes; the limit is {MEASUREMENTS_FILE_MAX_BYTES}"
    )]
    TooLarge { bytes: usize },
    #[error(
        "{MEASUREMENTS_FILE_PATH} entry {index} carries a `job` key, which grammar version 1 \
         does not support: a dispatch runs the repo's whole CI steps list"
    )]
    JobKeyNotSupported { index: usize },
    /// Malformed TOML, a missing key, a non-string value, or an unknown key
    /// (the parser is strict). The message is the TOML parser's full report,
    /// including the line and column.
    #[error("{MEASUREMENTS_FILE_PATH} is malformed: {0}")]
    Malformed(String),
    #[error("{MEASUREMENTS_FILE_PATH} declares {count} measurements; the limit is {MEASUREMENTS_MAX_ENTRIES}")]
    TooManyEntries { count: usize },
    #[error(
        "{MEASUREMENTS_FILE_PATH} entry {index}: `{}` must be non-blank, at most \
         {RUN_OUTCOME_LABEL_MAX_BYTES} bytes, without control characters or \
         leading/trailing whitespace", field.as_str()
    )]
    InvalidLabel {
        index: usize,
        field: MeasurementField,
    },
    #[error(
        "{MEASUREMENTS_FILE_PATH} entry {index}: `name` must be 1..={RUN_OUTCOME_METRIC_NAME_MAX_BYTES} \
         bytes of [A-Za-z0-9_.:-], not only dots"
    )]
    InvalidName { index: usize },
    #[error("{MEASUREMENTS_FILE_PATH} entry {index} repeats the measurement name `{name}`")]
    DuplicateName { index: usize, name: String },
}

/// Parse and validate the text of `.qontinui/measurements.toml`.
pub fn parse_measurements(text: &str) -> Result<MeasurementsFile, MeasurementDeclError> {
    if text.len() > MEASUREMENTS_FILE_MAX_BYTES {
        return Err(MeasurementDeclError::TooLarge { bytes: text.len() });
    }

    // One parse on the accepting path, straight into the strict types, so a
    // schema error (unknown or missing key) keeps the parser's line/column.
    // Only on failure is the text re-read as a plain table, to name the v1
    // `job` refusal rather than report it as just another unknown key.
    let file: MeasurementsFile = match toml::from_str(text) {
        Ok(file) => file,
        Err(e) => {
            if let Some(index) = job_key_index(text) {
                return Err(MeasurementDeclError::JobKeyNotSupported { index });
            }
            return Err(MeasurementDeclError::Malformed(e.to_string()));
        }
    };

    if file.measurements.len() > MEASUREMENTS_MAX_ENTRIES {
        return Err(MeasurementDeclError::TooManyEntries {
            count: file.measurements.len(),
        });
    }

    let mut names = BTreeSet::new();
    for (index, m) in file.measurements.iter().enumerate() {
        if !is_valid_metric_name(&m.name) || m.name.bytes().all(|b| b == b'.') {
            return Err(MeasurementDeclError::InvalidName { index });
        }
        for (field, value) in [
            (MeasurementField::Source, &m.source),
            (MeasurementField::Subject, &m.subject),
            (MeasurementField::Task, &m.task),
        ] {
            if !is_valid_run_outcome_label(value) {
                return Err(MeasurementDeclError::InvalidLabel { index, field });
            }
        }
        if !names.insert(m.name.as_str()) {
            return Err(MeasurementDeclError::DuplicateName {
                index,
                name: m.name.clone(),
            });
        }
    }

    Ok(file)
}

/// The index of the first `[[measurements]]` entry carrying a `job` key, when
/// the text is well-formed TOML.
fn job_key_index(text: &str) -> Option<usize> {
    let raw: toml::Table = toml::from_str(text).ok()?;
    match raw.get("measurements")? {
        toml::Value::Array(entries) => entries
            .iter()
            .position(|e| e.as_table().is_some_and(|t| t.contains_key(JOB_KEY))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
[[measurements]]
name = "unit-suite"
source = "cargo-test"
subject = "qontinui-types"
task = "unit"

[[measurements]]
name = "e2e"
source = "playwright"
subject = "qontinui-web"
task = "smoke"
"#;

    fn entry(name: &str, extra: &str) -> String {
        format!(
            "[[measurements]]\nname = \"{name}\"\nsource = \"s\"\nsubject = \"x\"\ntask = \"t\"\n{extra}\n"
        )
    }

    #[test]
    fn a_valid_file_parses_in_order() {
        let file = parse_measurements(VALID).expect("valid");
        assert_eq!(file.measurements.len(), 2);
        assert_eq!(
            file.measurements[0],
            MeasurementDecl {
                name: "unit-suite".into(),
                source: "cargo-test".into(),
                subject: "qontinui-types".into(),
                task: "unit".into(),
            }
        );
        assert_eq!(file.get("e2e").map(|m| m.task.as_str()), Some("smoke"));
        assert!(file.get("absent").is_none());
    }

    #[test]
    fn an_empty_file_declares_nothing() {
        assert_eq!(parse_measurements("").unwrap(), MeasurementsFile::default());
    }

    #[test]
    fn the_snapshot_serializes_to_the_four_labels() {
        let file = parse_measurements(VALID).unwrap();
        let v = serde_json::to_value(&file.measurements[0]).unwrap();
        assert_eq!(
            v,
            serde_json::json!({
                "name": "unit-suite", "source": "cargo-test",
                "subject": "qontinui-types", "task": "unit"
            })
        );
    }

    #[test]
    fn an_unknown_entry_key_is_refused() {
        let err = parse_measurements(&entry("a", "timeout = \"5m\"")).unwrap_err();
        assert!(
            matches!(&err, MeasurementDeclError::Malformed(m) if m.contains("timeout")),
            "{err:?}"
        );
    }

    #[test]
    fn an_unknown_top_level_key_is_refused() {
        let text = format!("version = 1\n{}", entry("a", ""));
        let err = parse_measurements(&text).unwrap_err();
        assert!(
            matches!(&err, MeasurementDeclError::Malformed(m) if m.contains("version")),
            "{err:?}"
        );
        // A foreign table (e.g. a ci.toml section pasted in) is refused too.
        let text = format!("{}\n[[steps]]\nrun = \"x\"\n", entry("a", ""));
        assert!(matches!(
            parse_measurements(&text),
            Err(MeasurementDeclError::Malformed(_))
        ));
    }

    #[test]
    fn a_job_key_is_refused_by_name() {
        let text = format!("{}{}", entry("a", ""), entry("b", "job = \"test\""));
        assert_eq!(
            parse_measurements(&text),
            Err(MeasurementDeclError::JobKeyNotSupported { index: 1 })
        );
    }

    #[test]
    fn a_missing_key_is_refused() {
        let text = "[[measurements]]\nname = \"a\"\nsource = \"s\"\nsubject = \"x\"\n";
        assert!(matches!(
            parse_measurements(text),
            Err(MeasurementDeclError::Malformed(m)) if m.contains("task")
        ));
    }

    #[test]
    fn an_over_long_label_is_refused_and_the_limit_is_accepted() {
        let at_limit = "t".repeat(RUN_OUTCOME_LABEL_MAX_BYTES);
        let over = "t".repeat(RUN_OUTCOME_LABEL_MAX_BYTES + 1);
        let ok = format!(
            "[[measurements]]\nname = \"a\"\nsource = \"s\"\nsubject = \"x\"\ntask = \"{at_limit}\"\n"
        );
        assert!(parse_measurements(&ok).is_ok());
        let bad = format!(
            "[[measurements]]\nname = \"a\"\nsource = \"s\"\nsubject = \"x\"\ntask = \"{over}\"\n"
        );
        assert_eq!(
            parse_measurements(&bad),
            Err(MeasurementDeclError::InvalidLabel {
                index: 0,
                field: MeasurementField::Task
            })
        );
    }

    fn with_source(source: &str) -> String {
        format!(
            "[[measurements]]\nname = \"a\"\nsource = \"{source}\"\nsubject = \"x\"\ntask = \"t\"\n"
        )
    }

    #[test]
    fn blank_padded_or_control_labels_are_refused() {
        for source in ["", " a", "a\\u0007"] {
            assert_eq!(
                parse_measurements(&with_source(source)),
                Err(MeasurementDeclError::InvalidLabel {
                    index: 0,
                    field: MeasurementField::Source
                }),
                "{source:?}"
            );
        }
        // A label may hold spaces inside; a name may not (below).
        assert!(parse_measurements(&with_source("cargo test")).is_ok());
    }

    #[test]
    fn a_name_must_be_a_slug() {
        assert!(parse_measurements(&entry("unit-suite.v1:x_Y9", "")).is_ok());
        // Dots are fine beside other characters; only an all-dot name is refused.
        assert!(parse_measurements(&entry(".a", "")).is_ok());
        let at_limit = "n".repeat(RUN_OUTCOME_METRIC_NAME_MAX_BYTES);
        assert!(parse_measurements(&entry(&at_limit, "")).is_ok());
        let over = "n".repeat(RUN_OUTCOME_METRIC_NAME_MAX_BYTES + 1);
        for name in [
            "",
            "unit suite",
            "ünit",
            "a/b",
            ".",
            "..",
            "...",
            " a",
            "a\\u0007",
            over.as_str(),
        ] {
            assert_eq!(
                parse_measurements(&entry(name, "")),
                Err(MeasurementDeclError::InvalidName { index: 0 }),
                "{name:?}"
            );
        }
    }

    #[test]
    fn a_malformed_report_keeps_the_line_and_column() {
        let text = format!("{}{}", entry("a", ""), entry("b", "timeout = \"5m\""));
        let Err(MeasurementDeclError::Malformed(m)) = parse_measurements(&text) else {
            panic!("expected Malformed");
        };
        assert!(m.contains("timeout") && m.contains("line 12"), "{m}");
        let Err(MeasurementDeclError::Malformed(m)) = parse_measurements("[[measurements]\n")
        else {
            panic!("expected Malformed");
        };
        assert!(m.contains("line 1"), "{m}");
    }

    #[test]
    fn a_duplicate_name_is_refused_at_the_second_index() {
        let text = format!("{}{}{}", entry("a", ""), entry("b", ""), entry("a", ""));
        assert_eq!(
            parse_measurements(&text),
            Err(MeasurementDeclError::DuplicateName {
                index: 2,
                name: "a".into()
            })
        );
    }

    #[test]
    fn size_and_count_caps() {
        // Exactly the entry cap is accepted.
        let at_cap: String = (0..MEASUREMENTS_MAX_ENTRIES)
            .map(|i| entry(&format!("m{i}"), ""))
            .collect();
        assert_eq!(
            parse_measurements(&at_cap).unwrap().measurements.len(),
            MEASUREMENTS_MAX_ENTRIES
        );
        let many: String = (0..=MEASUREMENTS_MAX_ENTRIES)
            .map(|i| entry(&format!("m{i}"), ""))
            .collect();
        assert_eq!(
            parse_measurements(&many),
            Err(MeasurementDeclError::TooManyEntries {
                count: MEASUREMENTS_MAX_ENTRIES + 1
            })
        );
        // Exactly the cap is accepted.
        let body = entry("a", "");
        let pad = MEASUREMENTS_FILE_MAX_BYTES - body.len() - 3;
        let exact = format!("{body}# {}\n", "x".repeat(pad));
        assert_eq!(exact.len(), MEASUREMENTS_FILE_MAX_BYTES);
        assert_eq!(parse_measurements(&exact).unwrap().measurements.len(), 1);
        let huge = format!("{exact}#");
        assert!(matches!(
            parse_measurements(&huge),
            Err(MeasurementDeclError::TooLarge { .. })
        ));
    }
}
