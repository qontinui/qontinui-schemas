/* eslint-disable */
/**
 * This file was automatically generated.
 * DO NOT MODIFY IT BY HAND. Regenerate with
 * `qontinui-runner/src-tauri/scripts/generate_types.sh`.
 */

import type { SkillPlaybookTrigger } from "./SkillPlaybookTrigger";
import type { SkillRef } from "./SkillRef";

export type SkillTemplate =
  | {
      kind: "single_step";
      step: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      kind: "multi_step";
      steps: {
        [k: string]: unknown;
      }[];
      [k: string]: unknown;
    }
  | {
      kind: "composition";
      skill_refs: SkillRef[];
      [k: string]: unknown;
    }
  | {
      /**
       * Full markdown content (the body after frontmatter).
       */
      content: string;
      kind: "playbook";
      /**
       * Trigger conditions for when this playbook should be included.
       */
      triggers: SkillPlaybookTrigger[];
      [k: string]: unknown;
    };
