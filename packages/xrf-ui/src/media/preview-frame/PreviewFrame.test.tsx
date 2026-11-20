import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { PreviewFrame } from "./PreviewFrame";

describe("PreviewFrame", () => {
  it("draws its content on the viewport backdrop, captioned beneath", () => {
    const { getByTestId, getByText } = renderWithTheme(
      <PreviewFrame data-testid={"preview"} id={"texture"} caption={"1024 × 1024 · DXT5"}>
        <span>picture</span>
      </PreviewFrame>
    );

    expect(getByTestId("preview-body")).toHaveClass("bg-viewport-backdrop");
    expect(getByTestId("preview-body")).toHaveAttribute("id", "texture-body");
    expect(getByTestId("preview-body")).toContainElement(getByText("picture"));
    expect(getByTestId("preview-footer")).toHaveTextContent("1024 × 1024 · DXT5");
  });

  it("draws the alpha checkerboard behind its content when asked", () => {
    const { getByTestId } = renderWithTheme(
      <PreviewFrame caption={"Reading…"} isCheckered>
        <span />
      </PreviewFrame>
    );

    expect(getByTestId("preview-frame-body")).toHaveClass("checkerboard");
    expect(getByTestId("preview-frame-body")).not.toHaveAttribute("id");
  });
});
