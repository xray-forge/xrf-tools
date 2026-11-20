import { default as createCache, EmotionCache } from "@emotion/cache";
import { Nullable, Optional } from "@xrf/types";

const INSERTION_POINT_NAME: string = "emotion-insertion-point";

/** One style cache per document: a cache never removes the rules it inserted, so a second one would duplicate them. */
const DOCUMENT_STYLE_CACHES: WeakMap<Document, EmotionCache> = new WeakMap();

/**
 * Puts MUI's styles at the top of `<head>`, ahead of the application's own stylesheet.
 *
 * Emotion appends to `<head>` by default, which lands its rules after the bundled stylesheet; a utility
 * would then lose to a component's own `sx` on the same property. Both sides are unlayered - MUI's
 * `enableCssLayer` cannot be used because jsdom does not implement cascade layers, and every
 * `getComputedStyle` assertion in the suite would read empty - so source order is what decides, and this
 * is what sets it. `tailwind.css` beside it keeps Tailwind's utilities out of a layer for the same
 * reason: an unlayered rule outranks every layer however the layers are ordered.
 *
 * @param document - The document the styles go into.
 * @returns The element emotion inserts its rules after.
 */
function getInsertionPoint(document: Document): HTMLElement {
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
 * The style cache every MUI component in a document injects through, made on first use.
 *
 * Shared by every theme root the document mounts: a cache per root would leave each unmounted root's copy of the
 * styles in `<head>`, and every later `getComputedStyle` would match all of those copies.
 *
 * @param document - The document the styles go into.
 * @returns That document's Emotion style cache.
 */
export function getDocumentStyleCache(document: Document): EmotionCache {
  const existing: Optional<EmotionCache> = DOCUMENT_STYLE_CACHES.get(document);

  if (existing) {
    return existing;
  }

  const created: EmotionCache = createCache({ key: "css", speedy: true, insertionPoint: getInsertionPoint(document) });

  DOCUMENT_STYLE_CACHES.set(document, created);

  return created;
}
