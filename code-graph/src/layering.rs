//! `Ξ_Layering` — the **deterministic, authored gate** over the resolved
//! `Ξ_AST` dependency graph.
//!
//! This module owns the *pure* layering verdict computation that both the runner
//! (its HTTP `/code-graph/layer-drift` observer) and coord (its merge gate) need:
//!
//! 1. the `qontinui-layers.toml` manifest parser ([`LayerManifest`]) — the authored
//!    declared side (the spec);
//! 2. the uncapped cross-module dependency edge set ([`cross_module_edges`]) derived
//!    from a resolved [`CodeGraph`];
//! 3. the layering-allowed-edge relation Φ + Tarjan cycle detection over that edge
//!    set, producing breaches/cycles ([`evaluate_phi`], [`evaluate_phi_manifest`],
//!    [`find_layer_cycles`]);
//! 4. the gate-grade verdict ([`compute_gate_verdict`] → [`LayerGateVerdict`]) — a
//!    `block`/`escalate`/`allow` recommendation a merge gate can enforce.
//!
//! It is **pure**: no model call, no `ai_provider`/`arch_classify` kernel, no Tauri.
//! The stochastic `Ξ_Arch` label-classifier (the advisory, no-manifest path) stays
//! in the host runner; only the deterministic authored-manifest path lives here.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::code_graph::{CodeGraph, ResolutionKind};

// ===========================================================================
// `qontinui-layers.toml` — the authored declared side
// ===========================================================================

/// The built-in downward allowed-edge relation (used when `[order].allowed` is empty).
const DEFAULT_ALLOWED: &[(&str, &str)] = &[
    ("ui", "api"),
    ("ui", "service"),
    ("api", "service"),
    ("service", "data"),
];

#[derive(Debug, Deserialize, Default)]
struct RawManifest {
    #[serde(default)]
    order: RawOrder,
    #[serde(default)]
    layers: BTreeMap<String, String>,
    #[serde(default)]
    exempt: RawExempt,
    #[serde(default)]
    carve_out: RawCarve,
}

#[derive(Debug, Deserialize)]
struct RawOrder {
    #[serde(default)]
    allowed: Vec<Vec<String>>,
    #[serde(default = "default_true")]
    #[allow(dead_code)] // documents intent; non-listed pairs are breaches regardless
    forbid_skip: bool,
}
impl Default for RawOrder {
    fn default() -> Self {
        RawOrder {
            allowed: Vec::new(),
            forbid_skip: true,
        }
    }
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Default)]
struct RawExempt {
    #[serde(default)]
    modules: Vec<String>,
    #[serde(default)]
    globs: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawCarve {
    #[serde(default)]
    edges: Vec<RawCarveRule>,
}

#[derive(Debug, Deserialize)]
struct RawCarveRule {
    from_glob: Option<String>,
    to_layer: Option<String>,
}

/// A glob/exact pattern reduced to its literal prefix + whether it is a glob.
#[derive(Debug, Clone)]
struct Pat {
    prefix: String,
    is_glob: bool,
}

impl Pat {
    fn parse(pattern: &str) -> Pat {
        if let Some(p) = pattern
            .strip_suffix("/**")
            .or_else(|| pattern.strip_suffix("/*"))
        {
            Pat {
                prefix: p.to_string(),
                is_glob: true,
            }
        } else {
            Pat {
                prefix: pattern.to_string(),
                is_glob: false,
            }
        }
    }

    fn matches(&self, module: &str) -> bool {
        if self.is_glob {
            module == self.prefix || module.starts_with(&format!("{}/", self.prefix))
        } else {
            module == self.prefix
        }
    }

    /// Specificity for "most-specific wins": prefix length, with an exact match
    /// beating a glob of the same length.
    fn specificity(&self) -> (usize, bool) {
        (self.prefix.len(), !self.is_glob)
    }
}

/// A carve-out rule: an edge whose `from` matches `from_glob` and whose `to` layer
/// equals `to_layer` is accepted (never a breach), recorded in the verdict.
#[derive(Debug, Clone)]
struct CarveRule {
    from: Pat,
    to_layer: String,
}

/// The parsed, resolved `qontinui-layers.toml` manifest.
#[derive(Debug, Clone, Default)]
pub struct LayerManifest {
    /// `[layers]` patterns sorted most-specific-first.
    layers: Vec<(Pat, String)>,
    /// `[exempt]` patterns (modules + globs).
    exempt: Vec<Pat>,
    carve: Vec<CarveRule>,
    allowed: Vec<(String, String)>,
}

impl LayerManifest {
    /// Load `<project_dir>/qontinui-layers.toml`. Returns `None` if the file is
    /// absent or unparseable (the caller then falls back to the `Ξ_Arch` kernel
    /// path). The manifest lives at the repo ROOT — NOT under `.qontinui/`, which is
    /// a runtime-state dir gitignored in some repos (an authored declared side must
    /// be committable).
    pub fn load(project_dir: &Path) -> Option<LayerManifest> {
        let path = project_dir.join("qontinui-layers.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        Self::parse(&content)
    }

    /// Parse manifest text (pure — the unit tests drive this directly).
    pub fn parse(content: &str) -> Option<LayerManifest> {
        let raw: RawManifest = toml::from_str(content).ok()?;

        let mut layers: Vec<(Pat, String)> = raw
            .layers
            .into_iter()
            .map(|(pat, layer)| (Pat::parse(&pat), layer))
            .collect();
        // Most-specific first: longer prefix wins; exact beats glob on a tie.
        layers.sort_by_key(|(pat, _)| std::cmp::Reverse(pat.specificity()));

        let mut exempt: Vec<Pat> = Vec::new();
        for m in raw.exempt.modules {
            exempt.push(Pat::parse(&m));
        }
        for g in raw.exempt.globs {
            exempt.push(Pat::parse(&g));
        }

        let carve: Vec<CarveRule> = raw
            .carve_out
            .edges
            .into_iter()
            .filter_map(|r| match (r.from_glob, r.to_layer) {
                (Some(from), Some(to_layer)) => Some(CarveRule {
                    from: Pat::parse(&from),
                    to_layer,
                }),
                _ => None,
            })
            .collect();

        let allowed: Vec<(String, String)> = if raw.order.allowed.is_empty() {
            DEFAULT_ALLOWED
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect()
        } else {
            raw.order
                .allowed
                .into_iter()
                .filter_map(|pair| match pair.as_slice() {
                    [a, b] => Some((a.clone(), b.clone())),
                    _ => None,
                })
                .collect()
        };

        Some(LayerManifest {
            layers,
            exempt,
            carve,
            allowed,
        })
    }

    /// The declared layer for a module, by the most-specific matching `[layers]`
    /// pattern. `None` if no pattern matches (a manifest coverage gap — the caller
    /// treats it as an `unknown`/unlabelled endpoint, lowering coverage honestly).
    pub fn layer_of(&self, module: &str) -> Option<&str> {
        self.layers
            .iter()
            .find(|(pat, _)| pat.matches(module))
            .map(|(_, layer)| layer.as_str())
    }

    /// Is this module exempt (a composition root / barrel)? Exempt modules are never
    /// judged by Φ and are excluded from cycle detection.
    pub fn is_exempt(&self, module: &str) -> bool {
        self.exempt.iter().any(|p| p.matches(module))
    }

    /// Is `a → b` an allowed layer edge? Intra-layer (`a == b`) is always allowed;
    /// otherwise membership in the (possibly overridden) allowed set.
    pub fn is_allowed(&self, a: &str, b: &str) -> bool {
        a == b
            || self
                .allowed
                .iter()
                .any(|(x, y)| x.as_str() == a && y.as_str() == b)
    }

    /// Is the edge `from` (→ a module of layer `to_layer`) an accepted carve-out?
    pub fn is_carved(&self, from: &str, to_layer: &str) -> bool {
        self.carve
            .iter()
            .any(|r| r.to_layer == to_layer && r.from.matches(from))
    }
}

// ===========================================================================
// Cross-module edge set (derived from a resolved CodeGraph)
// ===========================================================================

/// The module directory for a repo-relative file path (the parent dir, or `"."`
/// for repo-root files).
pub fn module_dir(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((dir, _)) if !dir.is_empty() => dir.to_string(),
        _ => ".".to_string(),
    }
}

/// The UNCAPPED cross-module (directory→directory) dependency digraph edges.
///
/// For every import that the resolver bound to an in-repo file (`resolved_target`
/// set, `resolution` neither `External` nor `Unresolved`), map both endpoints to
/// their module dir; keep the edge only when it crosses a module boundary
/// (`from != to`). This is the FULL edge set — the layering Φ predicate
/// (`Ξ_Layering`) MUST use this, never a capped summary, or a cross-layer breach
/// edge could be silently dropped. The `BTreeSet` de-duplicates + orders the edges
/// deterministically.
pub fn cross_module_edges(graph: &CodeGraph) -> BTreeSet<(String, String)> {
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
    for imp in &graph.imports {
        let target = match &imp.resolved_target {
            Some(t) => t,
            None => continue,
        };
        // Only honestly-resolved in-repo edges; External/Unresolved are not
        // cross-module dependency edges.
        if matches!(
            imp.resolution,
            ResolutionKind::External | ResolutionKind::Unresolved
        ) {
            continue;
        }
        let from = module_dir(&imp.from_file);
        let to = module_dir(target);
        if from != to {
            edges.insert((from, to));
        }
    }
    edges
}

// ===========================================================================
// Φ — the layering-allowed-edge relation (pure)
// ===========================================================================

/// The structural layers Φ reasons over. `utility` and `unknown` are deliberately
/// NOT here — they're handled by the carve-out, not the allowed-edge relation.
const STRUCTURAL_LAYERS: &[&str] = &["ui", "api", "service", "data"];

/// The built-in allowed-edge relation over the STRUCTURAL layers (used by the
/// advisory/non-manifest path). An edge `a → b` is allowed iff intra-layer
/// (`a == b`) or a sanctioned downward dependency.
fn is_allowed(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    const ALLOWED_CORE: &[(&str, &str)] = &[
        ("ui", "api"),
        ("ui", "service"),
        ("api", "service"),
        ("service", "data"),
    ];
    ALLOWED_CORE.contains(&(a, b))
}

/// Is this a structural layer Φ reasons over (`ui/api/service/data`)?
fn is_structural(layer: &str) -> bool {
    STRUCTURAL_LAYERS.contains(&layer)
}

/// Classification of a single edge under Φ + the carve-out.
#[derive(Debug, Clone, PartialEq, Eq)]
enum EdgeVerdict {
    /// Both endpoints structural + allowed (intra-layer or sanctioned downward).
    Clean,
    /// An endpoint is `utility` (import-by-anyone) — judged, clean, never a breach.
    CarvedUtility,
    /// An endpoint is unlabelled or `"unknown"` — NOT judged (lowers coverage).
    CarvedUnknown,
    /// An endpoint is an `exempt` module (composition root/barrel) — judged, clean,
    /// never a breach; also excluded from cycle detection.
    CarvedExempt,
    /// The edge matches a manifest `[carve_out]` rule — an accepted exception.
    CarvedException,
    /// Both endpoints structural, edge violates Φ — a layer breach.
    Breach {
        from_layer: String,
        to_layer: String,
    },
}

/// Evaluate one edge `(from, to)` under the built-in Φ + carve-out, given the labels.
fn classify_edge(from: &str, to: &str, layer_of: &HashMap<String, String>) -> EdgeVerdict {
    let la = layer_of.get(from).map(|s| s.as_str());
    let lb = layer_of.get(to).map(|s| s.as_str());

    let la = match la {
        Some(l) if l != "unknown" => l,
        _ => return EdgeVerdict::CarvedUnknown,
    };
    let lb = match lb {
        Some(l) if l != "unknown" => l,
        _ => return EdgeVerdict::CarvedUnknown,
    };

    if la == "utility" || lb == "utility" {
        return EdgeVerdict::CarvedUtility;
    }

    if is_allowed(la, lb) {
        EdgeVerdict::Clean
    } else {
        EdgeVerdict::Breach {
            from_layer: la.to_string(),
            to_layer: lb.to_string(),
        }
    }
}

/// Φ for one edge under the AUTHORED manifest: exempt endpoints and `[carve_out]`
/// matches are accepted (never breaches); otherwise the structural Φ with the
/// manifest's allowed-edge relation.
fn classify_edge_manifest(
    from: &str,
    to: &str,
    layer_of: &HashMap<String, String>,
    manifest: &LayerManifest,
) -> EdgeVerdict {
    if manifest.is_exempt(from) || manifest.is_exempt(to) {
        return EdgeVerdict::CarvedExempt;
    }
    let la = match layer_of.get(from).map(|s| s.as_str()) {
        Some(l) if l != "unknown" => l,
        _ => return EdgeVerdict::CarvedUnknown,
    };
    let lb = match layer_of.get(to).map(|s| s.as_str()) {
        Some(l) if l != "unknown" => l,
        _ => return EdgeVerdict::CarvedUnknown,
    };
    if la == "utility" || lb == "utility" {
        return EdgeVerdict::CarvedUtility;
    }
    if manifest.is_carved(from, lb) {
        return EdgeVerdict::CarvedException;
    }
    if manifest.is_allowed(la, lb) {
        EdgeVerdict::Clean
    } else {
        EdgeVerdict::Breach {
            from_layer: la.to_string(),
            to_layer: lb.to_string(),
        }
    }
}

/// A layer breach: a single dependency edge that violates Φ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Breach {
    pub from: String,
    pub to: String,
    pub from_layer: String,
    pub to_layer: String,
}

/// The outcome of running the built-in Φ over every edge.
pub struct PhiResult {
    pub breaches: Vec<Breach>,
    pub judged_edges: usize,
    pub carved_utility_or_unknown: usize,
}

/// Run the built-in Φ over the full (uncapped) edge set. Pure — feed it a fixture
/// edge set + a stub `layer_of` and it is fully testable without the model.
pub fn evaluate_phi(
    edges: &BTreeSet<(String, String)>,
    layer_of: &HashMap<String, String>,
) -> PhiResult {
    let mut breaches = Vec::new();
    let mut judged = 0usize;
    let mut carved = 0usize;
    for (from, to) in edges {
        match classify_edge(from, to, layer_of) {
            EdgeVerdict::Clean => judged += 1,
            EdgeVerdict::CarvedUtility
            | EdgeVerdict::CarvedExempt
            | EdgeVerdict::CarvedException => {
                judged += 1;
                carved += 1;
            }
            EdgeVerdict::CarvedUnknown => carved += 1,
            EdgeVerdict::Breach {
                from_layer,
                to_layer,
            } => {
                judged += 1;
                breaches.push(Breach {
                    from: from.clone(),
                    to: to.clone(),
                    from_layer,
                    to_layer,
                });
            }
        }
    }
    breaches.sort_by(|x, y| x.from.cmp(&y.from).then_with(|| x.to.cmp(&y.to)));
    PhiResult {
        breaches,
        judged_edges: judged,
        carved_utility_or_unknown: carved,
    }
}

/// The manifest-path Φ result, with the carve-out breakdown for the verdict body.
pub struct GatePhi {
    pub breaches: Vec<Breach>,
    pub judged: usize,
    pub exempt: usize,
    pub exception: usize,
    pub utility: usize,
    pub unknown: usize,
}

/// Run the AUTHORED-manifest Φ over the full (uncapped) edge set.
pub fn evaluate_phi_manifest(
    edges: &BTreeSet<(String, String)>,
    layer_of: &HashMap<String, String>,
    manifest: &LayerManifest,
) -> GatePhi {
    let mut g = GatePhi {
        breaches: Vec::new(),
        judged: 0,
        exempt: 0,
        exception: 0,
        utility: 0,
        unknown: 0,
    };
    for (from, to) in edges {
        match classify_edge_manifest(from, to, layer_of, manifest) {
            EdgeVerdict::Clean => g.judged += 1,
            EdgeVerdict::CarvedUtility => {
                g.judged += 1;
                g.utility += 1;
            }
            EdgeVerdict::CarvedExempt => {
                g.judged += 1;
                g.exempt += 1;
            }
            EdgeVerdict::CarvedException => {
                g.judged += 1;
                g.exception += 1;
            }
            EdgeVerdict::CarvedUnknown => g.unknown += 1,
            EdgeVerdict::Breach {
                from_layer,
                to_layer,
            } => {
                g.judged += 1;
                g.breaches.push(Breach {
                    from: from.clone(),
                    to: to.clone(),
                    from_layer,
                    to_layer,
                });
            }
        }
    }
    g.breaches
        .sort_by(|x, y| x.from.cmp(&y.from).then_with(|| x.to.cmp(&y.to)));
    g
}

/// Label every module node in `edges` from the manifest's `[layers]` globs. Modules
/// with no matching glob are omitted (treated as unlabelled → a manifest coverage
/// gap that lowers coverage honestly).
pub fn label_from_manifest(
    edges: &BTreeSet<(String, String)>,
    manifest: &LayerManifest,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (a, b) in edges {
        for m in [a, b] {
            if !out.contains_key(m) {
                if let Some(layer) = manifest.layer_of(m) {
                    out.insert(m.clone(), layer.to_string());
                }
            }
        }
    }
    out
}

// ===========================================================================
// Cycle detection — Tarjan SCC over the full digraph (pure)
// ===========================================================================

/// A cross-layer dependency cycle: a strongly-connected component (>1 member) that
/// spans ≥2 distinct STRUCTURAL layers (per the labels, ignoring utility/unknown
/// members).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerCycle {
    pub members: Vec<String>,
}

/// Tarjan SCC state over an integer-indexed adjacency list.
struct Tarjan<'a> {
    adj: &'a [Vec<usize>],
    index: Vec<Option<usize>>,
    lowlink: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    next_index: usize,
    sccs: Vec<Vec<usize>>,
}

impl<'a> Tarjan<'a> {
    fn new(adj: &'a [Vec<usize>]) -> Self {
        let n = adj.len();
        Tarjan {
            adj,
            index: vec![None; n],
            lowlink: vec![0; n],
            on_stack: vec![false; n],
            stack: Vec::new(),
            next_index: 0,
            sccs: Vec::new(),
        }
    }

    fn run(mut self) -> Vec<Vec<usize>> {
        for v in 0..self.adj.len() {
            if self.index[v].is_none() {
                self.strongconnect(v);
            }
        }
        self.sccs
    }

    fn strongconnect(&mut self, v: usize) {
        self.index[v] = Some(self.next_index);
        self.lowlink[v] = self.next_index;
        self.next_index += 1;
        self.stack.push(v);
        self.on_stack[v] = true;

        for i in 0..self.adj[v].len() {
            let w = self.adj[v][i];
            match self.index[w] {
                None => {
                    self.strongconnect(w);
                    self.lowlink[v] = self.lowlink[v].min(self.lowlink[w]);
                }
                Some(w_idx) => {
                    if self.on_stack[w] {
                        self.lowlink[v] = self.lowlink[v].min(w_idx);
                    }
                }
            }
        }

        if self.index[v] == Some(self.lowlink[v]) {
            let mut component = Vec::new();
            loop {
                let w = self.stack.pop().unwrap();
                self.on_stack[w] = false;
                component.push(w);
                if w == v {
                    break;
                }
            }
            self.sccs.push(component);
        }
    }
}

/// Find cross-layer cycles via Tarjan's SCC over the full edge digraph. Pure —
/// testable from a fixture edge set + stub labels.
pub fn find_layer_cycles(
    edges: &BTreeSet<(String, String)>,
    layer_of: &HashMap<String, String>,
) -> Vec<LayerCycle> {
    let mut node_set: BTreeSet<&str> = BTreeSet::new();
    for (from, to) in edges {
        node_set.insert(from.as_str());
        node_set.insert(to.as_str());
    }
    let nodes: Vec<&str> = node_set.into_iter().collect();
    let index_of: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, n)| (*n, i)).collect();

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for (from, to) in edges {
        adj[index_of[from.as_str()]].push(index_of[to.as_str()]);
    }

    let sccs = Tarjan::new(&adj).run();

    let mut cycles = Vec::new();
    for comp in sccs {
        if comp.len() < 2 {
            continue;
        }
        let mut members: Vec<String> = comp.iter().map(|&i| nodes[i].to_string()).collect();
        let distinct_structural: BTreeSet<&str> = members
            .iter()
            .filter_map(|m| layer_of.get(m).map(|s| s.as_str()))
            .filter(|&l| is_structural(l))
            .collect();
        if distinct_structural.len() >= 2 {
            members.sort();
            cycles.push(LayerCycle { members });
        }
    }
    cycles.sort_by(|a, b| a.members.cmp(&b.members));
    cycles
}

// ===========================================================================
// Verdict — the gate-grade output a merge gate enforces
// ===========================================================================

/// The coord wire token for the top-level `drift_class`, by precedence: any cycle
/// → `divergent`; else any breach → `in_place`; else any unjudged edge → `unknown`;
/// else → `none`.
pub fn drift_class(
    breaches: usize,
    cycles: usize,
    judged_edges: usize,
    total_edges: usize,
) -> &'static str {
    if cycles > 0 {
        "divergent"
    } else if breaches > 0 {
        "in_place"
    } else if judged_edges < total_edges {
        "unknown"
    } else {
        "none"
    }
}

/// The authored gate recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GateRec {
    /// A real layering violation against the authored spec (any breach/cycle).
    Block,
    /// The manifest has coverage gaps (unlabelled edges → can't fully verify).
    Escalate,
    /// Clean.
    Allow,
}

impl GateRec {
    /// `block` on any breach/cycle; `escalate` on a coverage gap; else `allow`.
    pub fn from_counts(breaches: usize, cycles: usize, unknown: usize) -> GateRec {
        if breaches > 0 || cycles > 0 {
            GateRec::Block
        } else if unknown > 0 {
            GateRec::Escalate
        } else {
            GateRec::Allow
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GateRec::Block => "block",
            GateRec::Escalate => "escalate",
            GateRec::Allow => "allow",
        }
    }

    /// A gate that refuses auto-merge (`block` or `escalate` — both fail-closed; in
    /// this codebase "escalate" routes to a specialist, it does not allow).
    pub fn is_blocking(self) -> bool {
        matches!(self, GateRec::Block | GateRec::Escalate)
    }
}

/// The authored gate recommendation as a wire string (`block`/`escalate`/`allow`).
/// Kept for the runner's byte-identical envelope output.
pub fn gate_recommendation(breaches: usize, cycles: usize, unknown: usize) -> &'static str {
    GateRec::from_counts(breaches, cycles, unknown).as_str()
}

/// The full gate-grade verdict over a resolved edge set + an authored manifest —
/// the typed value a merge gate (coord `pr_merge/predicate.rs`) enforces on.
#[derive(Debug, Clone, Serialize)]
pub struct LayerGateVerdict {
    pub gate: GateRec,
    pub drift_class: String,
    pub breaches: Vec<Breach>,
    pub cycles: Vec<LayerCycle>,
    pub coverage: f64,
    pub total_edges: usize,
    pub judged_edges: usize,
    pub unknown_edges: usize,
}

/// Compute the authored-manifest gate verdict over `edges`. Cycles run over the
/// exempt-filtered digraph so a composition root no longer anchors a mega-SCC. This
/// is the deterministic entry point both the runner gate envelope and coord's merge
/// gate share.
pub fn compute_gate_verdict(
    edges: &BTreeSet<(String, String)>,
    manifest: &LayerManifest,
) -> LayerGateVerdict {
    let layer_of = label_from_manifest(edges, manifest);
    let total_edges = edges.len();
    let phi = evaluate_phi_manifest(edges, &layer_of, manifest);

    let cycle_edges: BTreeSet<(String, String)> = edges
        .iter()
        .filter(|(a, b)| !manifest.is_exempt(a) && !manifest.is_exempt(b))
        .cloned()
        .collect();
    let cycles = find_layer_cycles(&cycle_edges, &layer_of);

    let coverage = if total_edges == 0 {
        0.0
    } else {
        (phi.judged as f64 / total_edges as f64).min(1.0)
    };
    let gate = GateRec::from_counts(phi.breaches.len(), cycles.len(), phi.unknown);
    let drift = drift_class(phi.breaches.len(), cycles.len(), phi.judged, total_edges);

    LayerGateVerdict {
        gate,
        drift_class: drift.to_string(),
        breaches: phi.breaches,
        cycles,
        coverage,
        total_edges,
        judged_edges: phi.judged,
        unknown_edges: phi.unknown,
    }
}

// ===========================================================================
// Tests (pure)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[order]
allowed = [["ui","api"],["ui","service"],["api","service"],["service","data"],["api","data"]]
forbid_skip = true

[layers]
"src-tauri/src/commands/**" = "api"
"src-tauri/src/mcp/**" = "api"
"src-tauri/src/database/**" = "data"
"src-tauri/src/util/**" = "utility"
"src-tauri/src/**" = "service"
"src/**" = "ui"

[exempt]
modules = ["src-tauri/src"]

[carve_out]
edges = [ { from_glob = "src-tauri/src/database/**", to_layer = "service", reason = "persist domain types" } ]
"#;

    fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(m, l)| (m.to_string(), l.to_string()))
            .collect()
    }

    fn edges(pairs: &[(&str, &str)]) -> BTreeSet<(String, String)> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn parses_and_resolves_most_specific_glob() {
        let m = LayerManifest::parse(SAMPLE).expect("parses");
        assert_eq!(m.layer_of("src-tauri/src/commands"), Some("api"));
        assert_eq!(m.layer_of("src-tauri/src/commands/recap"), Some("api"));
        assert_eq!(m.layer_of("src-tauri/src/mcp"), Some("api"));
        assert_eq!(m.layer_of("src-tauri/src/database/pg"), Some("data"));
        assert_eq!(m.layer_of("src-tauri/src/util"), Some("utility"));
        assert_eq!(m.layer_of("src-tauri/src/orchestrator"), Some("service"));
        assert_eq!(m.layer_of("src/components"), Some("ui"));
        assert_eq!(m.layer_of("vendor/thirdparty"), None);
    }

    #[test]
    fn exempt_matches_root_only() {
        let m = LayerManifest::parse(SAMPLE).unwrap();
        assert!(m.is_exempt("src-tauri/src"));
        assert!(!m.is_exempt("src-tauri/src/commands"));
    }

    #[test]
    fn order_override_sanctions_api_to_data() {
        let m = LayerManifest::parse(SAMPLE).unwrap();
        assert!(m.is_allowed("ui", "api"));
        assert!(m.is_allowed("service", "data"));
        assert!(
            m.is_allowed("api", "data"),
            "the [order] override sanctions api→data"
        );
        assert!(
            m.is_allowed("service", "service"),
            "intra-layer always allowed"
        );
        assert!(!m.is_allowed("data", "service"));
        assert!(!m.is_allowed("ui", "data"));
        assert!(!m.is_allowed("service", "api"));
    }

    #[test]
    fn carve_out_matches_from_glob_and_to_layer() {
        let m = LayerManifest::parse(SAMPLE).unwrap();
        assert!(m.is_carved("src-tauri/src/database/pg", "service"));
        assert!(!m.is_carved("src-tauri/src/database/pg", "api"));
        assert!(!m.is_carved("src-tauri/src/orchestrator", "service"));
    }

    #[test]
    fn default_order_when_omitted() {
        let m = LayerManifest::parse("[layers]\n\"a/**\" = \"api\"\n").unwrap();
        assert!(m.is_allowed("api", "service"));
        assert!(
            !m.is_allowed("api", "data"),
            "no override → api→data is a breach"
        );
    }

    #[test]
    fn empty_or_garbage() {
        assert!(LayerManifest::parse("").is_some());
        assert!(LayerManifest::parse("not = [valid").is_none());
    }

    #[test]
    fn module_dir_and_basename() {
        assert_eq!(module_dir("src/api/auth.ts"), "src/api");
        assert_eq!(module_dir("main.rs"), ".");
    }

    #[test]
    fn gate_verdict_blocks_on_a_manifest_breach() {
        let m = LayerManifest::parse(SAMPLE).unwrap();
        // data → api is a breach (no allowed edge; not carved).
        let e = edges(&[("src-tauri/src/database/pg", "src-tauri/src/commands/x")]);
        let v = compute_gate_verdict(&e, &m);
        assert_eq!(v.gate, GateRec::Block);
        assert_eq!(v.breaches.len(), 1);
        assert_eq!(v.breaches[0].from_layer, "data");
        assert_eq!(v.breaches[0].to_layer, "api");
        assert_eq!(v.drift_class, "in_place");
        assert!(v.gate.is_blocking());
    }

    #[test]
    fn gate_verdict_allows_a_clean_downward_edge() {
        let m = LayerManifest::parse(SAMPLE).unwrap();
        // api → service is sanctioned.
        let e = edges(&[("src-tauri/src/commands/x", "src-tauri/src/orchestrator")]);
        let v = compute_gate_verdict(&e, &m);
        assert_eq!(v.gate, GateRec::Allow);
        assert!(v.breaches.is_empty());
        assert!(!v.gate.is_blocking());
    }

    #[test]
    fn gate_verdict_escalates_on_a_coverage_gap() {
        let m = LayerManifest::parse(SAMPLE).unwrap();
        // one endpoint unlabelled by the manifest → unknown → escalate (no breach).
        let e = edges(&[("src-tauri/src/commands/x", "vendor/thirdparty")]);
        let v = compute_gate_verdict(&e, &m);
        assert_eq!(v.gate, GateRec::Escalate);
        assert_eq!(v.unknown_edges, 1);
        assert!(v.gate.is_blocking());
    }

    #[test]
    fn built_in_phi_flags_an_upward_breach() {
        // service → ui is not allowed under the built-in relation.
        let e = edges(&[("a", "b")]);
        let l = labels(&[("a", "service"), ("b", "ui")]);
        let phi = evaluate_phi(&e, &l);
        assert_eq!(phi.breaches.len(), 1);
    }

    #[test]
    fn cycles_need_two_distinct_structural_layers() {
        let e = edges(&[("a", "b"), ("b", "a")]);
        let l = labels(&[("a", "api"), ("b", "service")]);
        let cycles = find_layer_cycles(&e, &l);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].members, vec!["a".to_string(), "b".to_string()]);
    }
}
