import { ClassValue, default as clsx } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";

import { Maybe, Optional } from "@/lib/types/general";

/**
 * @param values - Id fragments; `false`, `null`, `undefined`, and `"` are skipped.
 * @returns The truthy fragments joined by a single space.
 */
export function tid(...values: Array<Maybe<string | false>>): Optional<string> {
  return values[0] ? values.filter(Boolean).join("-") : undefined;
}

/**
 * @param values - Id fragments; `false`, `null`, `undefined`, and `"` are skipped.
 * @returns The truthy fragments joined by a single space.
 */
export function uid(...values: Array<Maybe<string | false>>): Optional<string> {
  return values[0] ? values.filter(Boolean).join("-") : undefined;
}

/**
 * The scales `core/theme/tailwind.css` adds, named again so conflicts between them resolve.
 */
const merge = extendTailwindMerge({
  extend: {
    theme: {
      container: ["reading", "reading-label", "content-state-description"],
      font: ["monospace"],
      leading: ["panel"],
      radius: ["surface", "control"],
      spacing: [
        "tree-row",
        "tree-icon",
        "tree-gap",
        "code-line",
        "header",
        "dialog",
        "panel-property",
        "panel-property-gap",
        "panel-content",
        "panel-section",
        "panel-section-gap",
        "content-state",
        "content-state-gap",
        "editor-action",
        "rail-button",
        "rail",
        "title-bar",
        "status-bar",
      ],
      text: ["tree-icon", "content-state-icon", "badge", "panel-action-icon", "editor-action-icon"],
    },
  },
});

/**
 * Combines class names, letting the last one win where two of them set the same property.
 *
 * @param values - Class name fragments; falsy values are skipped.
 * @returns The merged class list.
 */
export function cn(...values: Array<ClassValue>): string {
  return merge(clsx(values));
}
