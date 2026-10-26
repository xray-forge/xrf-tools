import { describe, expect, it } from "@jest/globals";

import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { IEditorStatusSegment, isSameStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";

const SEGMENT: IEditorStatusSegment = {
  id: "memory",
  text: "Backend 260 MB",
  details: [
    { label: "Webview", value: "1.1 GB" },
    { label: "GPU", value: "480 MB", isNested: true },
  ],
};

describe("isSameStatusSegment", () => {
  it("compares plain segments by their text", () => {
    expect(isSameStatusSegment("3 sectors", "3 sectors")).toBe(true);
    expect(isSameStatusSegment("3 sectors", "4 sectors")).toBe(false);
  });

  it("tells a plain segment from a detailed one with the same text", () => {
    expect(isSameStatusSegment(SEGMENT.text, SEGMENT)).toBe(false);
  });

  it("treats a detailed segment rebuilt with the same figures as the same", () => {
    expect(
      isSameStatusSegment(SEGMENT, {
        ...SEGMENT,
        details: SEGMENT.details.map((it: IEditorStatusDetail): IEditorStatusDetail => ({ ...it })),
      })
    ).toBe(true);
  });

  it("treats an absent nesting flag as not nested", () => {
    expect(
      isSameStatusSegment(
        { ...SEGMENT, details: [{ label: "Webview", value: "1.1 GB", isNested: false }, SEGMENT.details[1]] },
        SEGMENT
      )
    ).toBe(true);
  });

  it.each([
    ["id", { ...SEGMENT, id: "heap" }],
    ["text", { ...SEGMENT, text: "Backend 261 MB" }],
    ["a value", { ...SEGMENT, details: [SEGMENT.details[0], { label: "GPU", value: "481 MB", isNested: true }] }],
    ["a label", { ...SEGMENT, details: [SEGMENT.details[0], { label: "Browser", value: "480 MB", isNested: true }] }],
    ["the nesting", { ...SEGMENT, details: [SEGMENT.details[0], { label: "GPU", value: "480 MB" }] }],
    ["the row count", { ...SEGMENT, details: [SEGMENT.details[0]] }],
  ])("tells segments apart by %s", (_: string, other: IEditorStatusSegment) => {
    expect(isSameStatusSegment(SEGMENT, other)).toBe(false);
  });
});
