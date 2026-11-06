/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

/**
 * The colour scheme a context renders in (`prefers-color-scheme`).
 */
export type ImageSlotTheme = "dark" | "light";

/**
 * One image placement's declared contract: what the image is for, where it
 * renders, and what it must and must not be.
 */
export interface ImageSlotSpec {
  /**
   * The image's text alternative.
   */
  alt:
    | {
        kind: "decorative";
      }
    | {
        kind: "required";
      }
    | {
        kind: "text";
        /**
         * The text alternative.
         */
        text: string;
      };
  aspect: ImageSlotAspect;
  /**
   * Who it is for: a reference to one of the tenant's `audience_profile`
   * prompt documents (`audience_profile/external-operator`).
   */
  audience: string;
  content: ImageSlotContent;
  contexts: ImageSlotContexts;
  focal: ImageSlotFocal;
  /**
   * The image's job, in plain sentences: what it must achieve for the person
   * looking at it. Judges derive yes/no questions from it.
   */
  job: string;
  /**
   * The asset bound to the slot, written by the asset pipeline and never by
   * the author. Absent until an asset is chosen; until then the UI renders
   * the slot's placeholder.
   */
  provenance?: ImageSlotProvenance | null;
  /**
   * The requirement this slot traces to, for requirement-level traceability:
   * one token with no whitespace, at most 256 characters.
   */
  requirement_ref?: string | null;
  /**
   * What the image is on its page.
   */
  role:
    | "hero"
    | "empty_state"
    | "feature"
    | "onboarding"
    | "og_card"
    | "background"
    | "avatar_fallback"
    | "scene"
    | "logo"
    | "screenshot";
  /**
   * The parts of the rendered image that page content is laid over, which
   * must therefore stay calm enough for that content to read.
   */
  safe_areas:
    | {
        kind: "derived";
      }
    | {
        kind: "explicit";
        /**
         * The areas, in the image's rendered box.
         *
         * @minItems 1
         */
        regions: [ImageSlotSafeArea, ...ImageSlotSafeArea[]];
      };
  /**
   * Stable slot id (`sessions.empty-state`): lowercase letters and digits in
   * segments joined by `.`, `-` or `_`, at most 128 characters. The UI marks
   * the rendered element `data-ui-bridge-image-slot="<slot>"`, and the asset
   * store, the candidate records and the judges key on it.
   */
  slot: string;
  style: ImageSlotStyle;
  /**
   * Whether the image may carry text, and which.
   */
  text_in_image:
    | {
        kind: "forbidden";
      }
    | {
        kind: "expected";
        /**
         * Its language, as a BCP 47 tag (`en`, `de-CH`).
         */
        locale: string;
        /**
         * The text as it must read.
         */
        text: string;
      }
    | {
        kind: "allowed";
      };
  /**
   * Where the thresholds of the slot's deterministic image checks come from.
   */
  thresholds:
    | {
        kind: "from_visual_system";
      }
    | {
        kind: "overrides";
        /**
         * Tolerated relative difference between the natural and rendered
         * aspect ratios: the `max_aspect_delta` of `image_not_distorted`.
         */
        max_aspect_delta?: number | null;
        /**
         * Encoded byte budget, at most 2^53 − 1: the `max_bytes` of
         * `image_weight_within`.
         */
        max_bytes?: number | null;
        /**
         * Ceiling on effective resolution, against oversized images: the
         * `max_effective` of `image_not_oversized`.
         */
        max_effective_resolution?: number | null;
        /**
         * Largest CIEDE2000 distance from the declared palette that still
         * counts as on-palette: the `max_delta_e` of `image_palette_within`.
         */
        max_palette_delta_e?: number | null;
        /**
         * Floor on effective resolution, natural ÷ (rendered × DPR): the
         * `min_effective` of `image_resolution_sufficient`.
         */
        min_effective_resolution?: number | null;
        /**
         * Least fraction (0 to 1) of the image's pixels that must be
         * on-palette: the `min_coverage` of `image_palette_within`.
         */
        min_palette_coverage?: number | null;
        /**
         * Least WCAG contrast ratio (1 to 21) of text laid over the image: the
         * `min_ratio` of `text_over_image_contrast`.
         */
        min_text_over_image_contrast?: number | null;
        /**
         * Least WCAG contrast ratio (1 to 21) of a transparent image's edge
         * and stroke pixels against what surrounds it: the `min_ratio` of
         * `transparent_image_legible`.
         */
        min_transparent_edge_contrast?: number | null;
        /**
         * Which percentile (0 to 100) of glyph-pixel contrast is compared with
         * that floor: the `percentile` of `text_over_image_contrast`.
         */
        text_over_image_percentile?: number | null;
      };
}
/**
 * The aspect ratios the rendered box may take.
 */
export interface ImageSlotAspect {
  /**
   * Other ratios a layout may give the box, in the same form. None may equal
   * `preferred` or another entry in value (`8/6` equals `4/3`). Required:
   * an author states `[]` when only `preferred` is allowed.
   */
  allowed: string[];
  /**
   * The ratio the box takes by default, as `W/H` with W and H whole numbers
   * from 1 to 9999 (`4/3`, `1200/630`). The placeholder reserves it.
   */
  preferred: string;
}
/**
 * What the image must and must not depict.
 */
export interface ImageSlotContent {
  /**
   * Things the image must depict, one per entry. May be empty.
   */
  must: string[];
  /**
   * Things the image must not depict, one per entry. May be empty.
   */
  must_not: string[];
}
/**
 * Every breakpoint × theme × device pixel ratio the image renders, is
 * captured and is judged at.
 */
export interface ImageSlotContexts {
  /**
   * Viewport widths in CSS pixels, strictly ascending (`[360, 768, 1280]`).
   *
   * @minItems 1
   */
  breakpoints: [number, ...number[]];
  /**
   * Device pixel ratios from 0.5 to 8, strictly ascending (`[1, 2]`).
   *
   * @minItems 1
   */
  dpr: [number, ...number[]];
  /**
   * Colour schemes, each at most once.
   *
   * @minItems 1
   */
  themes: [ImageSlotTheme, ...ImageSlotTheme[]];
}
/**
 * Where the subject sits in the image, and whether every crop must keep it.
 */
export interface ImageSlotFocal {
  /**
   * True when every rendered crop (object-fit and object-position in every
   * context) must keep the whole focal region visible.
   */
  must_survive_crop: boolean;
  region: ImageSlotRegion;
}
/**
 * The subject's region in the image's natural frame. An image with no
 * single subject, such as a texture, declares the whole frame with
 * `must_survive_crop: false`.
 */
export interface ImageSlotRegion {
  /**
   * Height, as a fraction of the frame's height.
   */
  height: number;
  /**
   * Width, as a fraction of the frame's width.
   */
  width: number;
  /**
   * Left edge, as a fraction of the frame's width.
   */
  x: number;
  /**
   * Top edge, as a fraction of the frame's height.
   */
  y: number;
}
/**
 * The asset bound to a slot, as the asset pipeline records it; never
 * authored. It describes the binding made for the spec whose content hash was
 * `spec_sha256`. When the spec's current content hash differs, the binding is
 * stale (the spec changed since), not invalid. A current binding also meets
 * the obligations stated on its fields and on [`ImageSlotRole`].
 */
export interface ImageSlotProvenance {
  /**
   * True when a model generated or edited the pixels. Such an asset, and
   * every derivative of it, carries the IPTC `trainedAlgorithmicMedia`
   * marker. Always true for `generated` and `edited`.
   */
  ai_generated: boolean;
  /**
   * The text alternative written for this asset. Required in a current
   * binding whenever the slot's `alt` is `required`.
   */
  alt_text?: string | null;
  /**
   * sha256 of the bound asset's original bytes: its content address in the
   * tenant's image store, which keys its generation record (prompt, seed,
   * weights, licence) and its judging record.
   */
  asset_sha256: string;
  /**
   * sha256 identifying the version of the exemplar set in force when the
   * asset was produced. Required in a current binding whenever the slot names
   * a set.
   */
  exemplars_sha256?: string | null;
  /**
   * sha256 of the imagery treatment's text as read when the asset was
   * produced. Required for `generated` and `edited`.
   */
  imagery_sha256?: string | null;
  /**
   * sha256 of the LoRA weights applied. Required in a current binding of a
   * model-made asset whenever the slot names a LoRA.
   */
  lora_sha256?: string | null;
  /**
   * Hash of the generation recipe. Required for `generated` and `edited`.
   */
  recipe_sha256?: string | null;
  /**
   * How the asset was obtained.
   */
  source:
    | "generated"
    | "edited"
    | "traced"
    | "authored"
    | "selected"
    | "captured"
    | "supplied";
  /**
   * The spec's content hash when the asset was bound: sha256 of the spec's
   * RFC 8785 canonical JSON with `provenance` left out.
   */
  spec_sha256: string;
}
/**
 * A rectangle in the image's RENDERED box (the element's box, where page
 * content is positioned), as fractions (0 to 1) of its width and height: `x`
 * and `y` are the top-left corner, `width` and `height` the size. It has a
 * positive area and lies inside the box.
 *
 * A distinct type from [`ImageSlotRegion`] because the two coordinate spaces
 * differ whenever object-fit crops or letterboxes the image.
 */
export interface ImageSlotSafeArea {
  /**
   * Height, as a fraction of the box's height.
   */
  height: number;
  /**
   * Width, as a fraction of the box's width.
   */
  width: number;
  /**
   * Left edge, as a fraction of the box's width.
   */
  x: number;
  /**
   * Top edge, as a fraction of the box's height.
   */
  y: number;
}
/**
 * The declared style the image follows.
 */
export interface ImageSlotStyle {
  /**
   * The exemplar set (approved and rejected examples) the image is compared
   * with, by id. Absent: none.
   */
  exemplars?: string | null;
  /**
   * The imagery treatment it follows: a section of one of the tenant's
   * `domain_spec` prompt documents (`domain_spec/visual-system#imagery`), or
   * a `decision_record` that settled a style fork.
   */
  imagery: string;
  /**
   * The house LoRA applied when generating, by its id in the image licence
   * manifest. Absent: none.
   */
  lora?: string | null;
}
