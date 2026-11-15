import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { renderWithTheme } from "#/testing/render";

import { EditorPanelProperty } from "./EditorPanelProperty";

const REFERENCE: string = "aaz\\actor\\act_aaz_svoboda4";

function renderProperty(value: string, isMonospace: boolean = false): HTMLElement {
  const render: RenderResult = renderWithTheme(
    <EditorPanelProperty label={"Texture"} value={value} isMonospace={isMonospace} />
  );

  return render.getByTestId("editor-panel-property");
}

describe("EditorPanelProperty", () => {
  it("shows the label and the value", () => {
    const property: HTMLElement = renderProperty("DXT5");

    expect(property.querySelector("dt")).toHaveTextContent("Texture");
    expect(property.querySelector("dd")).toHaveTextContent("DXT5");
  });

  it("stacks the label above a full-width value", () => {
    const property: HTMLElement = renderProperty(REFERENCE);
    const value: HTMLElement = property.querySelector("dd") as HTMLElement;

    expect(property).toHaveStyle({ flexDirection: "column" });
    expect(value).toHaveStyle({ width: "100%", minWidth: 0, textAlign: "start" });
  });

  it("sets the label in a column beside the value when inline", () => {
    const { getByTestId }: RenderResult = renderWithTheme(
      <EditorPanelProperty label={"Version"} value={"18"} isInline />
    );
    const property: HTMLElement = getByTestId("editor-panel-property");

    expect(property).toHaveClass("grid");
    expect(property).toHaveClass("sm:grid-cols-[var(--container-reading-label)_1fr]");
    expect(property).not.toHaveStyle({ flexDirection: "column" });
  });

  it("offers a path its separators to break on", () => {
    // `overflow-wrap` alone splits a name wherever the line runs out, which is how `act_aaz_svoboda4` ends up across
    // two lines at no meaningful point. The separators come first, so a path folds where a reader would fold it.
    const property: HTMLElement = renderProperty(REFERENCE, true);

    expect(property.querySelectorAll("wbr")).toHaveLength(3);
    expect(property).toHaveTextContent(REFERENCE);
  });

  it("leaves a value with nothing to break on alone", () => {
    const property: HTMLElement = renderProperty("act_aaz_svoboda4", true);

    expect(property.querySelectorAll("wbr")).toHaveLength(0);
    expect(property).toHaveTextContent("act_aaz_svoboda4");
  });

  it("renders a value that is not text at all", () => {
    // Chips and badges go through here too, and they carry their own layout.
    const render: RenderResult = renderWithTheme(
      <EditorPanelProperty label={"Bump"} value={<span data-testid={"chip"}>Flat</span>} isMonospace />
    );

    expect(render.getByTestId("chip")).toBeInTheDocument();
  });

  it("qualifies the value with its caption", () => {
    const { getByTestId }: RenderResult = renderWithTheme(
      <EditorPanelProperty label={"Sectors"} value={"38"} caption={"Portals join them"} />
    );
    const value: HTMLElement = getByTestId("editor-panel-property").querySelector("dd") as HTMLElement;

    expect(value).toHaveTextContent("38");
    expect(value).toHaveTextContent("Portals join them");
  });

  it("links a value that addresses something outside the application", () => {
    const { getByRole }: RenderResult = renderWithTheme(
      <EditorPanelProperty label={"Repository"} value={"xray-forge/xrf-tools"} href={"https://example.com/repo"} />
    );
    const link: HTMLElement = getByRole("link", { name: "xray-forge/xrf-tools" });

    expect(link).toHaveAttribute("href", "https://example.com/repo");
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noreferrer");
  });
});
