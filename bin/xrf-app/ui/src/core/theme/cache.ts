import { default as createCache, EmotionCache } from "@emotion/cache";
import { Nullable, Optional } from "@xrf/types";

const INSERTION_POINT_NAME: string = "emotion-insertion-point";

/**
 * Puts MUI's styles at the top of `<head>`, ahead of the application's own stylesheet.
 *
 * Emotion appends to `<head>` by default, which lands its rules after the bundled stylesheet; a utility
 * would then lose to a component's own `sx` on the same property. Both sides are unlayered - MUI's
 * `enableCssLayer` cannot be used because jsdom does not implement cascade layers, and every
 * `getComputedStyle` assertion in the suite would read empty - so source order is what decides, and this
 * is what sets it. `core/theme/tailwind.css` keeps Tailwind's utilities out of a layer for the same
 * reason: an unlayered rule outranks every layer however the layers are ordered.
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
