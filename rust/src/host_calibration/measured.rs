//! The measured manifest: absence is UNKNOWN, and the manifest says WHICH
//! absence.
//!
//! A calibration fact is in exactly one of three states, and the wire words
//! are the computers plan's (`2026-09-30-the-fleet-machine-is-not-a-first-class-coord-entity…`
//! §3.4), so a calibration sample and a host-lane sample read the same way:
//!
//! * `measured` — the probe ran and produced the value beside it;
//! * `not_supported` — this platform has no such instrument (PSI on Windows).
//!   Permanent, and needs no action;
//! * `unavailable` — the platform has the instrument but this read failed or
//!   was not taken (an unreadable file, a probe over its time budget). This is
//!   the UNKNOWN arm; `unknown` is accepted as an input alias for it.
//!
//! Neither failure word is ever a `0`. A `0` would read as "no pressure" —
//! served policy `verification-and-evidence`
//! `unknown-must-not-render-as-a-default`.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One axis's state in a measured manifest. The variant names are the wire
/// words, so `MeasuredState::Measured` is deliberate.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeasuredState {
    /// The probe ran and produced a value.
    Measured,
    /// The platform has no such instrument.
    NotSupported,
    /// The instrument exists but this read failed or was not taken (UNKNOWN).
    #[serde(alias = "unknown")]
    Unavailable,
}

impl MeasuredState {
    /// The wire word.
    pub fn as_str(self) -> &'static str {
        match self {
            MeasuredState::Measured => "measured",
            MeasuredState::NotSupported => "not_supported",
            MeasuredState::Unavailable => "unavailable",
        }
    }

    /// `measured` when the read produced something, else `unavailable`.
    pub fn from_read<T>(value: &Option<T>) -> Self {
        if value.is_some() {
            MeasuredState::Measured
        } else {
            MeasuredState::Unavailable
        }
    }
}

/// A flat axis-name → state map, the shape the computers plan's samples carry.
pub type MeasuredManifest = BTreeMap<String, MeasuredState>;

/// A value that is either measured, or one of the two named absences.
///
/// Serialized adjacently tagged — `{"state":"measured","value":…}`,
/// `{"state":"not_supported"}`, `{"state":"unavailable"}` — so a reader can
/// never take a missing value for a zero. It defaults to `unavailable`: a
/// field nobody filled in is UNKNOWN, not measured-empty.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum Measured<T> {
    /// The probe produced this value.
    Measured(T),
    /// The platform has no such instrument.
    NotSupported,
    /// The read failed or was not taken (UNKNOWN).
    #[default]
    #[serde(alias = "unknown")]
    Unavailable,
}

impl<T> Measured<T> {
    /// `Measured(v)` for `Some(v)`, `Unavailable` for `None` — a parser that
    /// returned nothing is a failed read, never a not-supported platform.
    pub fn from_read(value: Option<T>) -> Self {
        match value {
            Some(v) => Measured::Measured(v),
            None => Measured::Unavailable,
        }
    }

    /// The manifest word for this value.
    pub fn state(&self) -> MeasuredState {
        match self {
            Measured::Measured(_) => MeasuredState::Measured,
            Measured::NotSupported => MeasuredState::NotSupported,
            Measured::Unavailable => MeasuredState::Unavailable,
        }
    }

    /// The value, when measured.
    pub fn value(&self) -> Option<&T> {
        match self {
            Measured::Measured(v) => Some(v),
            _ => None,
        }
    }

    /// Consume into the value, when measured.
    pub fn into_value(self) -> Option<T> {
        match self {
            Measured::Measured(v) => Some(v),
            _ => None,
        }
    }

    /// True only for a measured value.
    pub fn is_measured(&self) -> bool {
        matches!(self, Measured::Measured(_))
    }

    /// Map a measured value; the two absences pass through unchanged.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Measured<U> {
        match self {
            Measured::Measured(v) => Measured::Measured(f(v)),
            Measured::NotSupported => Measured::NotSupported,
            Measured::Unavailable => Measured::Unavailable,
        }
    }

    /// Borrowing view.
    pub fn as_ref(&self) -> Measured<&T> {
        match self {
            Measured::Measured(v) => Measured::Measured(v),
            Measured::NotSupported => Measured::NotSupported,
            Measured::Unavailable => Measured::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_states_serialize_distinctly_and_never_as_zero() {
        let m: Measured<f64> = Measured::Measured(0.0);
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            r#"{"state":"measured","value":0.0}"#
        );
        let n: Measured<f64> = Measured::NotSupported;
        assert_eq!(
            serde_json::to_string(&n).unwrap(),
            r#"{"state":"not_supported"}"#
        );
        let u: Measured<f64> = Measured::default();
        assert_eq!(
            serde_json::to_string(&u).unwrap(),
            r#"{"state":"unavailable"}"#
        );
    }

    #[test]
    fn unknown_is_accepted_as_the_unavailable_alias() {
        let u: Measured<u64> = serde_json::from_str(r#"{"state":"unknown"}"#).unwrap();
        assert_eq!(u, Measured::Unavailable);
        let s: MeasuredState = serde_json::from_str(r#""unknown""#).unwrap();
        assert_eq!(s, MeasuredState::Unavailable);
    }

    #[test]
    fn from_read_maps_none_to_unavailable_not_not_supported() {
        assert_eq!(Measured::<u8>::from_read(None), Measured::Unavailable);
        assert_eq!(Measured::from_read(Some(3u8)).value(), Some(&3));
        assert_eq!(MeasuredState::from_read(&Some(1)), MeasuredState::Measured);
        assert_eq!(
            MeasuredState::from_read::<u8>(&None),
            MeasuredState::Unavailable
        );
    }

    #[test]
    fn map_preserves_absences() {
        assert_eq!(
            Measured::<u8>::NotSupported.map(|v| v * 2),
            Measured::NotSupported
        );
        assert_eq!(
            Measured::Measured(2u8).map(|v| v * 2),
            Measured::Measured(4)
        );
        assert_eq!(MeasuredState::NotSupported.as_str(), "not_supported");
    }
}
