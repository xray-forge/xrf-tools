import { describe, expect, it } from "@jest/globals";

import { ButtonBase } from "#/button";
import { renderWithTheme } from "#/testing/render";

/** Every rule the style caches have inserted into the document. */
function countInsertedRules(): number {
  return Array.from(document.head.querySelectorAll("style[data-emotion]")).reduce(
    (count: number, tag: Element) => count + ((tag as HTMLStyleElement).sheet?.cssRules.length ?? 0),
    0
  );
}

describe("ThemeRoot", () => {
  it("keeps one copy of the styles however often a root mounts", () => {
    const first = renderWithTheme(<ButtonBase>styled</ButtonBase>);
    const inserted: number = countInsertedRules();

    first.unmount();
    renderWithTheme(<ButtonBase>styled</ButtonBase>);

    expect(inserted).toBeGreaterThan(0);
    expect(countInsertedRules()).toBe(inserted);
  });
});
