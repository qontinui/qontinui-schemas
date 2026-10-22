//! Spec model — the typed frontmatter of a specification artifact.
//!
//! Plan `2026-10-10-spec-front-end-phase-6-specification-model-and-studio`
//! Phase 2 (design decision S1). A spec artifact — a request, requirement,
//! interface mapping or story in the plan library (`agent.work_artifacts`) —
//! is Markdown with a YAML frontmatter block. The frontmatter holds the
//! machine-read fields (stable ID, EARS pattern, acceptance criteria with
//! their verification method, source citations, external references, the
//! template the artifact was authored from); the prose stays prose. These
//! types are the one contract the studio (TypeScript), the web backend and
//! the spec service (Python) and the runner (Rust) read that block through.
//!
//! ## The EARS vocabulary lives here
//!
//! [`EarsPattern`] is the runner's former `EarsCategory`, moved here with its
//! unwanted-behaviour variant, its renderer and its classifier, so the
//! studio's EARS assistance and the runner's criteria→steps synthesis share
//! one vocabulary. The runner re-exports it under its old name, as it does
//! [`CriterionVerification`] (the runner's former `VerificationMethod`,
//! renamed here because [`crate::verification::VerificationMethod`] already
//! owns that binding name).
//!
//! The keywords the classifier reads are DATA — [`EARS_LEXICONS`], one
//! [`EarsLexicon`] per [`SpecLanguage`] with a [`EarsTemplate`] per pattern —
//! rather than literals inside the classifier, so a language is added by
//! adding a table and a pattern a table leaves out is never classified.
//!
//! ## Wire format
//!
//! Unlike the crate's JSON DTOs, frontmatter keys are `snake_case`: this is an
//! authored YAML document, read and written by people in the studio, and the
//! `prompt_template` seeds it is rendered from (coord's
//! `prompt_templates/spec_*.md`) already spell their keys that way. The
//! variant tag is `kind`, carrying the work-artifact kind
//! (`request` | `requirement` | `interface_mapping` | `story`).
//! `deny_unknown_fields` is omitted so a reader tolerates a newer document.
//!
//! IDs are the plan library's stable `spec_ref`s (`REQ-0042`, `STY-0007`, …),
//! minted by qontinui-web on create; nothing here mints or validates one.
//!
//! ## Parsing and rendering
//!
//! [`parse`] and [`render`] read and write a whole artifact body. They need a
//! YAML implementation, which the rest of this crate does not, so they sit
//! behind the opt-in `spec-yaml` feature: a consumer that only needs the types
//! (the runner, coord) adds nothing to its lockfile. `render(parse(x)) == x`
//! byte-for-byte for canonical input — a body [`render`] produced.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===========================================================================
// EARS — the requirement-sentence patterns
// ===========================================================================

/// EARS requirement pattern (Easy Approach to Requirements Syntax).
/// Categorizes criteria by their trigger/condition pattern for better
/// verification step generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum EarsPattern {
    /// Always true: "The system shall [action]"
    Ubiquitous,
    /// Triggered by event: "When [event], the system shall [action]"
    EventDriven,
    /// Depends on state: "While [state], the system shall [action]"
    StateDriven,
    /// Feature toggle: "Where [feature], the system shall [action]"
    Optional,
    /// Combined conditions: "While [state], when [event], the system shall [action]"
    Complex,
    /// Unwanted behaviour: "If [trigger], then the system shall [response]" —
    /// how the system must respond to an undesired situation (a failure, an
    /// invalid input, a lost connection). Serialized `unwanted_behavior`; the
    /// British spelling EARS itself uses is accepted on input.
    #[serde(alias = "unwanted_behaviour")]
    UnwantedBehavior,
    /// Not applicable (legacy criteria without EARS classification)
    #[default]
    #[serde(other)]
    NotApplicable,
}

impl EarsPattern {
    /// Every classified pattern, in the order the prompts list them.
    /// `NotApplicable` is the absence of a classification, not a pattern.
    pub const CLASSIFIED: [EarsPattern; 6] = [
        EarsPattern::Ubiquitous,
        EarsPattern::EventDriven,
        EarsPattern::StateDriven,
        EarsPattern::Optional,
        EarsPattern::Complex,
        EarsPattern::UnwantedBehavior,
    ];

    /// The wire name — identical to the serde serialization — or `"n/a"` for
    /// an unclassified criterion.
    pub fn as_str(&self) -> &'static str {
        match self {
            EarsPattern::Ubiquitous => "ubiquitous",
            EarsPattern::EventDriven => "event_driven",
            EarsPattern::StateDriven => "state_driven",
            EarsPattern::Optional => "optional",
            EarsPattern::Complex => "complex",
            EarsPattern::UnwantedBehavior => "unwanted_behavior",
            EarsPattern::NotApplicable => "n/a",
        }
    }

    /// Whether a criterion of this pattern names a trigger (an event, a
    /// state, a feature or an unwanted condition) that verification has to
    /// set up before it can assert anything.
    pub fn requires_trigger(&self) -> bool {
        !matches!(self, EarsPattern::Ubiquitous | EarsPattern::NotApplicable)
    }

    /// Render a criterion as its English EARS sentence:
    /// `If <trigger>, then the <system> shall <action>` and so on.
    ///
    /// A missing trigger on a pattern that needs one renders as
    /// `(trigger not specified)` rather than collapsing into the ubiquitous
    /// form, so an incomplete criterion reads as incomplete. `Complex` takes
    /// its trigger verbatim — it already carries its own "While …, when …".
    /// `NotApplicable` renders the action alone.
    pub fn render(&self, system: &str, trigger: Option<&str>, action: &str) -> String {
        let trigger = trigger
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .unwrap_or("(trigger not specified)");
        match self {
            EarsPattern::Ubiquitous => format!("The {system} shall {action}"),
            EarsPattern::EventDriven => format!("When {trigger}, the {system} shall {action}"),
            EarsPattern::StateDriven => format!("While {trigger}, the {system} shall {action}"),
            EarsPattern::Optional => format!("Where {trigger}, the {system} shall {action}"),
            EarsPattern::Complex => format!("{trigger}, the {system} shall {action}"),
            EarsPattern::UnwantedBehavior => {
                format!("If {trigger}, then the {system} shall {action}")
            }
            EarsPattern::NotApplicable => action.to_string(),
        }
    }

    /// Classify a requirement sentence in any language [`EARS_LEXICONS`]
    /// covers: the first lexicon that recognises a pattern wins, and a
    /// sentence none recognises is [`EarsPattern::NotApplicable`].
    pub fn classify(sentence: &str) -> EarsPattern {
        EARS_LEXICONS
            .iter()
            .map(|lexicon| Self::classify_in(sentence, lexicon))
            .find(|pattern| *pattern != EarsPattern::NotApplicable)
            .unwrap_or(EarsPattern::NotApplicable)
    }

    /// Classify a requirement sentence by its EARS template in one language,
    /// ignoring case, leading whitespace and a list prefix (`1.`, `2)`, `-`,
    /// `*`, `•`). Keywords are read from `lexicon`; the English ones are
    /// named below.
    ///
    /// Every pattern requires a `shall` word — a sentence with none is not an
    /// EARS requirement. The sentence is split into comma-separated clauses;
    /// the opening keyword is the first word of the first clause, and it must
    /// be followed by at least one trigger word.
    ///
    /// * **If** → [`EarsPattern::UnwantedBehavior`] when a `then` completes
    ///   the template, in one of two places:
    ///   - **with commas:** some clause after the first, up to and including
    ///     the clause holding `shall`, OPENS with `then` —
    ///     `If <trigger>, then the <system> shall …`, and also
    ///     `If <trigger>, then <adverbial>, the <system> shall …`;
    ///   - **without commas** (the whole sentence is one clause): a `then` at
    ///     the third word or later, not directly after a conjunction, with
    ///     at least one word between it and `shall`. Nothing checks that what
    ///     follows the `then` is a subject, so a comma-less
    ///     "If the user logs in then logs out the session shall end" still
    ///     reads as unwanted behaviour.
    ///
    ///   Otherwise an `If` sentence is [`EarsPattern::NotApplicable`] — e.g.
    ///   "If the user logs in and then logs out, the session shall end",
    ///   whose `then` sits inside the trigger clause instead of opening one.
    /// * **While** → [`EarsPattern::Complex`] when a `when` comes before
    ///   `shall`: opening a later clause (`While <state>, when <event>, …`),
    ///   or inside the first clause at the third word or later, not directly
    ///   after a conjunction, with a word after it
    ///   (`While <state> when <event>, the <system> shall …`, with or
    ///   without the second comma). A `when` after `shall` belongs to the
    ///   response, so "While uploading, the system shall show progress when
    ///   asked" stays [`EarsPattern::StateDriven`], as does any other
    ///   `While` sentence.
    /// * **When** → [`EarsPattern::EventDriven`]; **Where** →
    ///   [`EarsPattern::Optional`]; **The** (with `shall` in the same
    ///   clause) → [`EarsPattern::Ubiquitous`];
    /// * anything else → [`EarsPattern::NotApplicable`].
    ///
    /// A pattern the lexicon carries no template for is never returned.
    /// This is a keyword reading of the template, not a parser: it is meant to
    /// suggest a pattern, never to overrule one an author set.
    pub fn classify_in(sentence: &str, lexicon: &EarsLexicon) -> EarsPattern {
        let lower = strip_list_prefix(sentence).to_lowercase();
        let clauses: Vec<Vec<&str>> = lower
            .split(',')
            .map(|clause| {
                clause
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| !w.is_empty())
                    .collect::<Vec<&str>>()
            })
            .filter(|words| !words.is_empty())
            .collect();
        let is_shall = |w: &str| lexicon.shall.contains(&w);
        let Some(shall_at) = clauses.iter().position(|c| c.iter().any(|w| is_shall(w))) else {
            return EarsPattern::NotApplicable;
        };
        let first = &clauses[0];
        // A trigger needs at least one word after its keyword.
        let has_trigger = first.len() >= 2 && !is_shall(first[1]);
        // `word` inside the FIRST clause: at the third word or later, not
        // directly after a conjunction, with at least one word after it and
        // before `shall` (or before the clause's end when `shall` is in a
        // later clause).
        let stop = first
            .iter()
            .position(|w| is_shall(w))
            .unwrap_or(first.len());
        let inline_in_first = |word: &str| -> bool {
            (2..stop).any(|i| {
                first[i] == word && !lexicon.conjunctions.contains(&first[i - 1]) && i + 1 < stop
            })
        };
        // A later clause, up to and including the `shall` clause, opened by `word`.
        let opens_later_clause = |word: &str| clauses[1..=shall_at].iter().any(|c| c[0] == word);
        let opener = first[0];
        let opened_by = |pattern: EarsPattern| {
            lexicon
                .template(pattern)
                .filter(|t| t.openers.contains(&opener))
        };

        if let Some(unwanted) = opened_by(EarsPattern::UnwantedBehavior) {
            if !has_trigger {
                return EarsPattern::NotApplicable;
            }
            let completed = unwanted.connective.is_some_and(|then| {
                opens_later_clause(then) || (shall_at == 0 && inline_in_first(then))
            });
            return if completed {
                EarsPattern::UnwantedBehavior
            } else {
                EarsPattern::NotApplicable
            };
        }
        if has_trigger {
            if let Some(complex) = opened_by(EarsPattern::Complex) {
                let when_before_shall = complex
                    .connective
                    .is_some_and(|when| inline_in_first(when) || opens_later_clause(when));
                if when_before_shall {
                    return EarsPattern::Complex;
                }
            }
            for pattern in [
                EarsPattern::StateDriven,
                EarsPattern::EventDriven,
                EarsPattern::Optional,
            ] {
                if opened_by(pattern).is_some() {
                    return pattern;
                }
            }
        }
        if shall_at == 0 && opened_by(EarsPattern::Ubiquitous).is_some() {
            return EarsPattern::Ubiquitous;
        }
        EarsPattern::NotApplicable
    }
}

/// Drop a leading list marker — `1.`, `12)`, `-`, `*` or `•` followed by
/// whitespace — and surrounding whitespace, so a numbered requirement list
/// classifies line by line.
fn strip_list_prefix(sentence: &str) -> &str {
    let trimmed = sentence.trim();
    let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
    let had_digits = rest.len() != trimmed.len();
    let after_marker = if had_digits {
        rest.strip_prefix('.').or_else(|| rest.strip_prefix(')'))
    } else {
        rest.strip_prefix('-')
            .or_else(|| rest.strip_prefix('*'))
            .or_else(|| rest.strip_prefix('•'))
    };
    match after_marker {
        Some(tail) if tail.starts_with(char::is_whitespace) => tail.trim_start(),
        _ => trimmed,
    }
}

/// A natural language a spec artifact is written in.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SpecLanguage {
    /// English.
    En,
    /// German.
    De,
}

/// One EARS pattern's keywords and sentence template in one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarsTemplate {
    pub pattern: EarsPattern,
    /// Lower-case words that may OPEN a sentence of this pattern. For
    /// `Ubiquitous` these are the subject's articles (`the`); for `Complex`
    /// the state openers it shares with `StateDriven`.
    pub openers: &'static [&'static str],
    /// The word that completes a two-part pattern: `then` for
    /// `UnwantedBehavior`, the event's `when` for `Complex`. `None` for the
    /// single-keyword patterns.
    pub connective: Option<&'static str>,
    /// The canonical sentence, placeholders in angle brackets.
    pub template: &'static str,
}

/// The EARS keywords and templates of one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarsLexicon {
    pub language: SpecLanguage,
    /// Lower-case modal verbs that make a sentence a requirement (`shall`).
    pub shall: &'static [&'static str],
    /// Lower-case conjunctions; a connective directly after one belongs to
    /// the trigger, not to the template.
    pub conjunctions: &'static [&'static str],
    /// One template per classified pattern this language recognises.
    pub templates: &'static [EarsTemplate],
}

impl EarsLexicon {
    /// This language's template for `pattern`, if it has one.
    pub fn template(&self, pattern: EarsPattern) -> Option<&'static EarsTemplate> {
        self.templates.iter().find(|t| t.pattern == pattern)
    }
}

/// English EARS keywords and templates.
pub const EARS_EN: EarsLexicon = EarsLexicon {
    language: SpecLanguage::En,
    shall: &["shall"],
    conjunctions: &["and", "or", "but"],
    templates: &[
        EarsTemplate {
            pattern: EarsPattern::Ubiquitous,
            openers: &["the"],
            connective: None,
            template: "The <system> shall <response>",
        },
        EarsTemplate {
            pattern: EarsPattern::EventDriven,
            openers: &["when"],
            connective: None,
            template: "When <trigger>, the <system> shall <response>",
        },
        EarsTemplate {
            pattern: EarsPattern::StateDriven,
            openers: &["while"],
            connective: None,
            template: "While <state>, the <system> shall <response>",
        },
        EarsTemplate {
            pattern: EarsPattern::Optional,
            openers: &["where"],
            connective: None,
            template: "Where <feature>, the <system> shall <response>",
        },
        EarsTemplate {
            pattern: EarsPattern::Complex,
            openers: &["while"],
            connective: Some("when"),
            template: "While <state>, when <trigger>, the <system> shall <response>",
        },
        EarsTemplate {
            pattern: EarsPattern::UnwantedBehavior,
            openers: &["if"],
            connective: Some("then"),
            template: "If <trigger>, then the <system> shall <response>",
        },
    ],
};

/// German EARS keywords and templates.
pub const EARS_DE: EarsLexicon = EarsLexicon {
    language: SpecLanguage::De,
    shall: &["muss", "müssen", "soll", "sollen"],
    conjunctions: &["und", "oder", "aber"],
    templates: &[
        EarsTemplate {
            pattern: EarsPattern::Ubiquitous,
            openers: &["das", "der", "die"],
            connective: None,
            template: "Das <System> muss <Reaktion>",
        },
        EarsTemplate {
            pattern: EarsPattern::EventDriven,
            openers: &["wenn", "sobald"],
            connective: None,
            template: "Wenn <Auslöser>, muss das <System> <Reaktion>",
        },
        EarsTemplate {
            pattern: EarsPattern::StateDriven,
            openers: &["während", "solange"],
            connective: None,
            template: "Während <Zustand>, muss das <System> <Reaktion>",
        },
        EarsTemplate {
            pattern: EarsPattern::Optional,
            openers: &["wo", "sofern"],
            connective: None,
            template: "Wo <Merkmal>, muss das <System> <Reaktion>",
        },
        EarsTemplate {
            pattern: EarsPattern::Complex,
            openers: &["während", "solange"],
            connective: Some("wenn"),
            template: "Während <Zustand>, wenn <Auslöser>, muss das <System> <Reaktion>",
        },
        EarsTemplate {
            pattern: EarsPattern::UnwantedBehavior,
            openers: &["falls"],
            connective: Some("dann"),
            template: "Falls <Auslöser>, dann muss das <System> <Reaktion>",
        },
    ],
};

/// Every language the classifier reads, in the order [`EarsPattern::classify`]
/// tries them.
pub const EARS_LEXICONS: &[EarsLexicon] = &[EARS_EN, EARS_DE];

// ===========================================================================
// Shared frontmatter parts
// ===========================================================================

/// How an acceptance criterion is verified.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum CriterionVerification {
    /// Shell command (exit code, stdout check)
    #[default]
    Command,
    /// UI Bridge SDK assertion
    UiBridge,
    /// Test runner (Playwright, pytest, etc.)
    Test,
    /// Cannot be automated — human review needed
    Manual,
}

/// One acceptance criterion of a requirement or story.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SpecAcceptanceCriterion {
    /// Stable within its artifact, e.g. `AC-1`.
    pub id: String,
    /// The observable condition, ideally an EARS sentence.
    pub text: String,
    /// How the criterion is checked. Required: a spec states it rather than
    /// inheriting a default.
    pub verification: CriterionVerification,
}

/// What a source citation points at.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceCitationKind {
    /// A code location (file, symbol).
    Code,
    /// A section of a document.
    DocSection,
    /// A ticket in an issue tracker.
    Ticket,
    /// An incident record.
    Incident,
    /// Another plan-library artifact.
    Artifact,
}

/// Evidence a requirement was derived from.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SourceCitation {
    pub kind: SourceCitationKind,
    /// Where it is: a path and symbol, a document and heading, a ticket key,
    /// an artifact's `spec_ref`.
    pub locator: String,
    /// The version cited (a commit, a document revision), when one applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// The same item in an external system (a Jira issue, say).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExternalRef {
    /// The external system, e.g. `jira`.
    pub system: String,
    /// The item's key in that system, e.g. `PAY-123`.
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// The `prompt_template` document (and its version) an artifact was authored
/// from — `spec-story` or a `spec-story-for-<lob>` variant, say — so a later
/// template edit is traceable to the artifacts it affects.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TemplateRef {
    pub name: String,
    pub version: u32,
}

// ===========================================================================
// Frontmatter per kind
// ===========================================================================

/// The frontmatter of a spec artifact, tagged by its work-artifact `kind`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpecFrontmatter {
    Request(RequestFrontmatter),
    Requirement(RequirementFrontmatter),
    InterfaceMapping(InterfaceMappingFrontmatter),
    Story(StoryFrontmatter),
}

impl SpecFrontmatter {
    /// The stable `spec_ref` of the artifact.
    pub fn id(&self) -> &str {
        match self {
            SpecFrontmatter::Request(f) => &f.id,
            SpecFrontmatter::Requirement(f) => &f.id,
            SpecFrontmatter::InterfaceMapping(f) => &f.id,
            SpecFrontmatter::Story(f) => &f.id,
        }
    }

    /// The work-artifact kind, as the `kind` tag spells it.
    pub fn kind(&self) -> &'static str {
        match self {
            SpecFrontmatter::Request(_) => "request",
            SpecFrontmatter::Requirement(_) => "requirement",
            SpecFrontmatter::InterfaceMapping(_) => "interface_mapping",
            SpecFrontmatter::Story(_) => "story",
        }
    }
}

/// An incoming request — the raw ask a requirement is derived from.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RequestFrontmatter {
    pub id: String,
    /// Where the request came from, e.g. `email`, `jira`, `workshop`.
    pub source_system: String,
    /// Its reference there (a message id, a ticket key).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    /// The role of the person asking — a role, never a name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requester_role: Option<String>,
    /// ISO 8601.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_at: Option<String>,
}

/// A requirement.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RequirementFrontmatter {
    pub id: String,
    #[serde(default)]
    pub ears: EarsPattern,
    /// May be empty: completeness is the spec quality gate's verdict, not a
    /// parse error.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<SpecAcceptanceCriterion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceCitation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_refs: Vec<ExternalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateRef>,
}

/// A field-level mapping between two systems.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct InterfaceMappingFrontmatter {
    pub id: String,
    pub source_system: String,
    pub target_system: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<MappingRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateRef>,
}

/// One row of an interface mapping.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct MappingRow {
    pub source_field: String,
    pub target_field: String,
    /// The transformation applied, in prose or a short expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    /// The business rule or requirement (`spec_ref`) the row implements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_ref: Option<String>,
}

/// A user story.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StoryFrontmatter {
    pub id: String,
    /// The `spec_ref`s of the requirements this story refines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refines: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<SpecAcceptanceCriterion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_refs: Vec<ExternalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateRef>,
}

// ===========================================================================
// Parse / render (feature `spec-yaml`)
// ===========================================================================

/// Why a body did not parse as a spec artifact.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpecParseError {
    /// The body does not open with a `---` line.
    #[error("a spec artifact must open with a `---` frontmatter fence")]
    MissingFrontmatter,
    /// The opening fence has no closing `---` line.
    #[error("the frontmatter block opened with `---` is never closed by a `---` line")]
    UnterminatedFrontmatter,
    /// The block is not valid frontmatter for any spec kind.
    #[error("the frontmatter does not parse: {0}")]
    Invalid(String),
}

/// The fence line that opens and closes the frontmatter block.
pub const FRONTMATTER_FENCE: &str = "---";

/// Split a body into `(frontmatter, prose)`: the frontmatter is everything
/// between an opening `---` first line and the next `---` line, the prose
/// everything after that closing line. LF and CRLF line endings both work.
pub fn split_frontmatter(body: &str) -> Result<(&str, &str), SpecParseError> {
    let mut lines = body.split_inclusive('\n');
    let open = lines.next().ok_or(SpecParseError::MissingFrontmatter)?;
    if trim_eol(open) != FRONTMATTER_FENCE {
        return Err(SpecParseError::MissingFrontmatter);
    }
    let start = open.len();
    let mut at = start;
    for line in lines {
        if trim_eol(line) == FRONTMATTER_FENCE {
            return Ok((&body[start..at], &body[at + line.len()..]));
        }
        at += line.len();
    }
    Err(SpecParseError::UnterminatedFrontmatter)
}

fn trim_eol(line: &str) -> &str {
    line.strip_suffix('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .unwrap_or(line)
}

/// Parse a spec artifact body into its frontmatter and its prose (the text
/// after the closing fence, verbatim).
#[cfg(any(feature = "spec-yaml", test))]
pub fn parse(body: &str) -> Result<(SpecFrontmatter, &str), SpecParseError> {
    let (frontmatter, prose) = split_frontmatter(body)?;
    let parsed =
        serde_yaml_ng::from_str(frontmatter).map_err(|e| SpecParseError::Invalid(e.to_string()))?;
    Ok((parsed, prose))
}

/// Render a spec artifact body: the frontmatter as a `---`-fenced YAML block,
/// then `prose` verbatim. The output is canonical: [`parse`] reads it back to
/// the same frontmatter and prose, and rendering that again is byte-identical.
#[cfg(any(feature = "spec-yaml", test))]
pub fn render(frontmatter: &SpecFrontmatter, prose: &str) -> String {
    let yaml = serde_yaml_ng::to_string(frontmatter)
        .expect("spec frontmatter is plain data and always serializes");
    format!("{FRONTMATTER_FENCE}\n{yaml}{FRONTMATTER_FENCE}\n{prose}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- EARS ----------------------------------------------------------

    /// Every template classifies as its own pattern, in its own language —
    /// the tables and the classifier cannot disagree.
    #[test]
    fn every_template_classifies_as_its_own_pattern() {
        let mut checked = 0;
        for lexicon in EARS_LEXICONS {
            for pattern in EarsPattern::CLASSIFIED {
                let template = lexicon.template(pattern).unwrap_or_else(|| {
                    panic!("{:?} has no {pattern:?} template", lexicon.language)
                });
                assert_eq!(
                    EarsPattern::classify_in(template.template, lexicon),
                    pattern,
                    "{:?}: {}",
                    lexicon.language,
                    template.template
                );
                checked += 1;
            }
        }
        assert_eq!(checked, EARS_LEXICONS.len() * EarsPattern::CLASSIFIED.len());
    }

    const ENGLISH: &[(&str, EarsPattern)] = &[
        (
            "The checkout service shall log every payment.",
            EarsPattern::Ubiquitous,
        ),
        (
            "When the user submits the form, the system shall send a receipt.",
            EarsPattern::EventDriven,
        ),
        (
            "While the account is locked, the system shall refuse sign-in.",
            EarsPattern::StateDriven,
        ),
        (
            "Where two-factor login is enabled, the system shall ask for a code.",
            EarsPattern::Optional,
        ),
        (
            "While offline, when the user saves, the app shall queue the change.",
            EarsPattern::Complex,
        ),
        (
            "If the payment gateway times out, then the system shall retry once.",
            EarsPattern::UnwantedBehavior,
        ),
        (
            "If the user logs in and then logs out, the session shall end.",
            EarsPattern::NotApplicable,
        ),
        (
            "While uploading, the system shall show progress when asked.",
            EarsPattern::StateDriven,
        ),
        ("1. The system shall start.", EarsPattern::Ubiquitous),
        ("The system starts quickly.", EarsPattern::NotApplicable),
    ];

    const GERMAN: &[(&str, EarsPattern)] = &[
        ("Das System muss jede Zahlung protokollieren.", EarsPattern::Ubiquitous),
        (
            "Wenn der Nutzer das Formular absendet, muss das System eine Quittung senden.",
            EarsPattern::EventDriven,
        ),
        (
            "Während das Konto gesperrt ist, muss das System die Anmeldung verweigern.",
            EarsPattern::StateDriven,
        ),
        (
            "Wo die Zwei-Faktor-Anmeldung aktiviert ist, muss das System einen Code abfragen.",
            EarsPattern::Optional,
        ),
        (
            "Während die App offline ist, wenn der Nutzer speichert, muss die App die Änderung vormerken.",
            EarsPattern::Complex,
        ),
        (
            "Falls das Zahlungsgateway nicht antwortet, dann muss das System einmal wiederholen.",
            EarsPattern::UnwantedBehavior,
        ),
        ("Das System startet schnell.", EarsPattern::NotApplicable),
    ];

    #[test]
    fn classifies_english_sentences() {
        for (sentence, expected) in ENGLISH {
            assert_eq!(EarsPattern::classify(sentence), *expected, "{sentence}");
            assert_eq!(
                EarsPattern::classify_in(sentence, &EARS_EN),
                *expected,
                "{sentence}"
            );
        }
    }

    #[test]
    fn classifies_german_sentences() {
        for (sentence, expected) in GERMAN {
            assert_eq!(EarsPattern::classify(sentence), *expected, "{sentence}");
            assert_eq!(
                EarsPattern::classify_in(sentence, &EARS_DE),
                *expected,
                "{sentence}"
            );
        }
    }

    /// Both language tables cover every classified pattern, each with a
    /// sentence above — so dropping a template (the plan's mutation check:
    /// the unwanted-behaviour one) fails a case rather than passing silently.
    #[test]
    fn every_pattern_has_a_case_in_both_languages() {
        for cases in [ENGLISH, GERMAN] {
            for pattern in EarsPattern::CLASSIFIED {
                assert!(
                    cases.iter().any(|(_, p)| *p == pattern),
                    "no case for {pattern:?}"
                );
            }
        }
    }

    #[test]
    fn a_lexicon_without_a_template_never_returns_its_pattern() {
        let templates: Vec<EarsTemplate> = EARS_EN
            .templates
            .iter()
            .copied()
            .filter(|t| t.pattern != EarsPattern::UnwantedBehavior)
            .collect();
        let without_unwanted = EarsLexicon {
            templates: Box::leak(templates.into_boxed_slice()),
            ..EARS_EN
        };
        assert_eq!(
            EarsPattern::classify_in(
                "If the payment gateway times out, then the system shall retry once.",
                &without_unwanted
            ),
            EarsPattern::NotApplicable
        );
    }

    #[test]
    fn ears_wire_names() {
        for pattern in EarsPattern::CLASSIFIED {
            assert_eq!(
                serde_json::to_value(pattern).unwrap(),
                serde_json::Value::String(pattern.as_str().to_string())
            );
        }
        let british: EarsPattern = serde_json::from_str("\"unwanted_behaviour\"").unwrap();
        assert_eq!(british, EarsPattern::UnwantedBehavior);
        let unknown: EarsPattern = serde_json::from_str("\"something_new\"").unwrap();
        assert_eq!(unknown, EarsPattern::NotApplicable);
    }

    #[test]
    fn renders_unwanted_behaviour() {
        assert_eq!(
            EarsPattern::UnwantedBehavior.render("system", Some("the disk is full"), "warn"),
            "If the disk is full, then the system shall warn"
        );
        assert_eq!(
            EarsPattern::EventDriven.render("system", None, "warn"),
            "When (trigger not specified), the system shall warn"
        );
    }

    // ---- parse / render ------------------------------------------------

    fn fixtures() -> Vec<(SpecFrontmatter, &'static str)> {
        vec![
            (
                SpecFrontmatter::Request(RequestFrontmatter {
                    id: "RQ-0001".into(),
                    source_system: "email".into(),
                    source_ref: Some("msg-17".into()),
                    requester_role: Some("product owner".into()),
                    received_at: Some("2026-10-01T09:00:00Z".into()),
                }),
                "# Faster refunds\n\nCustomers wait too long for refunds.\n",
            ),
            (
                SpecFrontmatter::Requirement(RequirementFrontmatter {
                    id: "REQ-0042".into(),
                    ears: EarsPattern::UnwantedBehavior,
                    acceptance_criteria: vec![SpecAcceptanceCriterion {
                        id: "AC-1".into(),
                        text: "If the gateway times out, then the system shall retry once.".into(),
                        verification: CriterionVerification::Test,
                    }],
                    sources: vec![SourceCitation {
                        kind: SourceCitationKind::DocSection,
                        locator: "refund-policy.md#timeouts".into(),
                        version: Some("v3".into()),
                    }],
                    external_refs: vec![ExternalRef {
                        system: "jira".into(),
                        key: "PAY-123".into(),
                        url: None,
                    }],
                    template: Some(TemplateRef {
                        name: "spec-functional-concept".into(),
                        version: 1,
                    }),
                }),
                "## Context\n\nRefunds go through the gateway.\n",
            ),
            (
                SpecFrontmatter::InterfaceMapping(InterfaceMappingFrontmatter {
                    id: "IFM-0003".into(),
                    source_system: "orders".into(),
                    target_system: "ledger".into(),
                    rows: vec![MappingRow {
                        source_field: "amount_cents".into(),
                        target_field: "amount".into(),
                        transform: Some("divide by 100".into()),
                        rule_ref: Some("REQ-0042".into()),
                    }],
                    template: None,
                }),
                "",
            ),
            (
                SpecFrontmatter::Story(StoryFrontmatter {
                    id: "STY-0007".into(),
                    refines: vec!["REQ-0042".into()],
                    acceptance_criteria: vec![],
                    external_refs: vec![],
                    template: Some(TemplateRef {
                        name: "spec-story-for-payments".into(),
                        version: 2,
                    }),
                }),
                "As a customer, I want my refund fast, so that I trust the shop.\n",
            ),
        ]
    }

    #[test]
    fn render_and_parse_round_trip_every_kind() {
        let fixtures = fixtures();
        assert_eq!(fixtures.len(), 4, "one fixture per kind");
        for (frontmatter, prose) in fixtures {
            let body = render(&frontmatter, prose);
            let (parsed, parsed_prose) = parse(&body).expect(&body);
            assert_eq!(parsed, frontmatter);
            assert_eq!(parsed_prose, prose);
            assert_eq!(
                render(&parsed, parsed_prose),
                body,
                "canonical body re-renders"
            );
            assert!(body.contains(&format!("kind: {}\n", frontmatter.kind())));
        }
    }

    #[test]
    fn a_requirement_without_acceptance_criteria_parses() {
        let (fm, prose) = parse("---\nkind: requirement\nid: REQ-0001\n---\nprose\n").unwrap();
        let SpecFrontmatter::Requirement(req) = fm else {
            panic!("not a requirement");
        };
        assert!(req.acceptance_criteria.is_empty());
        assert_eq!(req.ears, EarsPattern::NotApplicable);
        assert_eq!(prose, "prose\n");
    }

    #[test]
    fn crlf_bodies_parse() {
        let (fm, prose) =
            parse("---\r\nkind: story\r\nid: STY-0001\r\n---\r\nAs a user\r\n").unwrap();
        assert_eq!(fm.id(), "STY-0001");
        assert_eq!(prose, "As a user\r\n");
    }

    #[test]
    fn malformed_bodies_are_refused() {
        assert_eq!(parse("no fence"), Err(SpecParseError::MissingFrontmatter));
        assert_eq!(parse(""), Err(SpecParseError::MissingFrontmatter));
        assert_eq!(
            parse("---\nkind: story\nid: STY-1\n"),
            Err(SpecParseError::UnterminatedFrontmatter)
        );
        assert!(matches!(
            parse("---\nkind: epic\nid: E-1\n---\n"),
            Err(SpecParseError::Invalid(_))
        ));
        assert!(matches!(
            parse("---\nkind: requirement\n---\n"),
            Err(SpecParseError::Invalid(_))
        ));
    }
}
