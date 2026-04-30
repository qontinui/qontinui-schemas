/**
 * Primitive IR shapes shared by states, transitions, and adapters.
 *
 * IR-only fields (provenance, metadata, effect) live here so that the runtime
 * adapter can identify and strip them in one place.
 *
 * See ADR-001 (qontinui-dev-notes/ui-bridge-redesign/section-1-foundations/)
 * for the rationale on each field.
 */

// ---------------------------------------------------------------------------
// Provenance — where an IR declaration came from
// ---------------------------------------------------------------------------

/**
 * Origin of an IR node. Set by the build plugin when extracting JSX wrappers,
 * by hand when authoring a JSON IR file directly, or by a generation pipeline.
 */
export interface IRProvenance {
  /** How this declaration was authored. */
  source: "hand-authored" | "build-plugin" | "ai-generated" | "migrated";
  /** Source file (relative to the build root). */
  file?: string;
  /** Line number in the source file (1-based). */
  line?: number;
  /** Column in the source file (1-based). */
  column?: number;
  /** Build-plugin version that produced this node, if applicable. */
  pluginVersion?: string;
}

// ---------------------------------------------------------------------------
// Metadata — semantic context routed through the useUIAnnotation store at runtime
// ---------------------------------------------------------------------------

/**
 * Human-authored semantic context for an IR node. Aligns with the existing
 * ElementAnnotation shape (ui-bridge/packages/ui-bridge/src/annotations/types.ts)
 * so the runtime SDK can write straight into the global annotation store
 * without parallel infrastructure.
 */
export interface IRMetadata {
  /** Short human-readable description of what this state/transition represents. */
  description?: string;
  /** What this state/transition is for, intent-wise. */
  purpose?: string;
  /** Tags for grouping, filtering, and search. */
  tags?: string[];
  /** IDs of related elements/states/transitions. */
  relatedElements?: string[];
  /** Free-form notes for nuance that doesn't fit description/purpose. */
  notes?: string;
}

// ---------------------------------------------------------------------------
// Effect — side-effect annotation on transitions
// ---------------------------------------------------------------------------

/**
 * Whether a transition is read-only, mutating, or destructive.
 *
 * - "read"        — query/navigate; no persistent state change.
 * - "write"       — modifies persistent state but is reversible (or has an undo).
 * - "destructive" — irreversible state change (delete, send, charge, deploy).
 *
 * Drives counterfactual analysis (section 6) and gates auto-regression
 * generation (section 9) — destructive transitions are excluded from
 * automatic walks.
 */
export type IREffect = "read" | "write" | "destructive";

// ---------------------------------------------------------------------------
// Cross-references between IR documents
// ---------------------------------------------------------------------------

/**
 * Pointer from one IR document to a state/transition declared elsewhere.
 * Used when a transition activates a state owned by another page's IR doc.
 */
export interface IRCrossRef {
  /** Document ID containing the referenced node. */
  doc: string;
  /** ID of the referenced state or transition within that document. */
  ref: string;
}

// ---------------------------------------------------------------------------
// Visual references (placeholder for section 8)
// ---------------------------------------------------------------------------

/**
 * Pointer to a captured visual reference (screenshot region, design token).
 * Optional companion data — populated by section 8's visual-fusion work.
 */
export interface IRVisualRef {
  /** Path or content-addressable ID of the captured asset. */
  asset: string;
  /** Capture region in viewport coordinates, if applicable. */
  region?: { x: number; y: number; width: number; height: number };
  /** Capture timestamp (epoch ms). */
  capturedAt?: number;
}
