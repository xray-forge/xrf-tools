import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverNote } from "./EditorPopoverNote";

describe("EditorPopoverNote", () => {
  it("reads as information by default, and as a warning in the warning tone", () => {
    const { getAllByTestId } = renderWithTheme(
      <>
        <EditorPopoverNote>Draws the level unlit</EditorPopoverNote>
        <EditorPopoverNote tone={"warning"}>Past the visibility mask</EditorPopoverNote>
      </>
    );
    const [info, warning] = getAllByTestId("editor-popover-note") as [HTMLElement, HTMLElement];

    expect(info).toHaveTextContent("Draws the level unlit");
    expect(info).toHaveClass("text-text-secondary");
    expect(warning).toHaveTextContent("Past the visibility mask");
    expect(warning).toHaveClass("text-warning");
  });
});
