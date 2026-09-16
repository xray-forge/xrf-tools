import { default as createCache, EmotionCache } from "@emotion/cache";

import { Nullable, Optional } from "@/lib/types/general";

const INSERTION_POINT_NAME: string = "emotion-insertion-point";

/**
 * Puts MUI's styles at the top of `<head>`, ahead of the application's own stylesheet.
 *
 * Emotion appends to `<head>` by default, which lands its rules after the bundled stylesheet and lets a
 * component's own styling beat a utility of equal specificity. MUI's `enableCssLayer` solves the same
 * problem with `@layer`, which cannot be used here: jsdom does not implement cascade layers, so every
 * rule inside one stops applying and each `getComputedStyle` assertion in the suite silently reads empty.
 */
function getInsertionPoint(): Optional<HTMLElement> {
  if (typeof document !== "object") {
    return undefined;
  }

  const existing: Nullable<HTMLElement> = document.querySelector(`meta[name="${INSERTION_POINT_NAME}"]`);

  if (existing) {
    return existing;
  }

  const created: HTMLMetaElement = document.createElement("meta");

  created.setAttribute("name", INSERTION_POINT_NAME);
  created.setAttribute("content", "");
  document.head.prepend(created);

  return created;
}

/**
 * The style cache every MUI component injects through.
 *
 * @returns The application's Emotion style cache.
 */
export function createApplicationStyleCache(): EmotionCache {
  return createCache({ key: "css", speedy: true, insertionPoint: getInsertionPoint() });
}
