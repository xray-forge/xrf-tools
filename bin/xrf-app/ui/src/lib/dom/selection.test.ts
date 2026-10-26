import { afterEach, describe, expect, it, jest } from "@jest/globals";

import { clearWindowTextSelection } from "@/lib/dom/selection";

/**
 * Selects the text of an element, the way a drag across a panel does.
 *
 * @param text - What to put on the page and select.
 * @returns The element holding it.
 */
function select(text: string): HTMLElement {
  const element: HTMLElement = document.createElement("p");

  element.textContent = text;
  document.body.appendChild(element);

  const range: Range = document.createRange();

  range.selectNodeContents(element);
  window.getSelection()?.removeAllRanges();
  window.getSelection()?.addRange(range);

  return element;
}

afterEach(() => {
  window.getSelection()?.removeAllRanges();
  document.body.innerHTML = "";
});

describe("clearWindowTextSelection", () => {
  it("drops what the window has selected", () => {
    select("a row a person dragged across");

    expect(window.getSelection()?.toString()).not.toBe("");

    clearWindowTextSelection();

    expect(window.getSelection()?.toString()).toBe("");
  });

  // Removing the ranges of an empty selection is harmless and still fires `selectionchange`, which anything listening
  // would hear as a change of nothing.
  it("leaves an empty selection alone", () => {
    const onChange = jest.fn();

    document.addEventListener("selectionchange", onChange);
    clearWindowTextSelection();
    document.removeEventListener("selectionchange", onChange);

    expect(onChange).not.toHaveBeenCalled();
  });
});
