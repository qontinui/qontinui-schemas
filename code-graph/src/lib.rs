//! # qontinui-code-graph
//!
//! Pure, filesystem-persistence-free codebase knowledge graph.
//!
//! This crate owns ALL graph *computation*: tree-sitter parsing of
//! TypeScript/JavaScript, Python, and Rust into a lightweight in-memory graph of
//! functions, classes, imports, and exports; deterministic, language-aware import
//! resolution; blast-radius analysis; and prompt formatting. It has NO coupling
//! to any host runner (no app-data paths, no Tauri, no runner-internal types).
//!
//! The incremental builder takes prior state as a parameter
//! ([`IncrementalInput`]) rather than reading/writing any cache file — the host
//! is responsible for persistence (load prior state, call
//! [`CodeGraph::build_incremental`], save the result).

pub mod code_graph;
pub mod import_resolver;
pub mod layering;

pub use code_graph::{
    compute_fingerprints, fingerprint_content, repo_hash, BlastRadius, CachedCodeGraph, ClassNode,
    CodeGraph, ExportNode, FileFingerprint, FileNode, FunctionNode, ImportEdge, IncrementalInput,
    ResolutionKind, RiskLevel,
};
pub use import_resolver::{ImportResolver, Resolution};
pub use layering::{
    compute_gate_verdict, cross_module_edges, module_dir, GateRec, LayerGateVerdict, LayerManifest,
};
