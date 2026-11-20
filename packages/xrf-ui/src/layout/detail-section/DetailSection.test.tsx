import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { DetailSection } from "./DetailSection";

describe("DetailSection", () => {
  it("states the title, its sentence and its content", () => {
    const { getByTestId, getByText } = renderWithTheme(
      <DetailSection title={"Archives"} description={"What each volume holds."} fact={"4 archives"}>
        <div data-testid={"rows"} />
      </DetailSection>
    );

    expect(getByTestId("detail-section")).toHaveTextContent("Archives");
    expect(getByText("What each volume holds.")).toBeInTheDocument();
    expect(getByText("4 archives")).toBeInTheDocument();
    expect(getByTestId("rows")).toBeInTheDocument();
  });

  it("puts content straight under a title that needs no sentence", () => {
    const { getByTestId, getByText } = renderWithTheme(
      <DetailSection title={"Signature"}>
        <div data-testid={"rows"} />
      </DetailSection>
    );

    const section: HTMLElement = getByTestId("detail-section");

    expect(section.children).toHaveLength(2);
    expect(getByText("Signature").parentElement).toHaveClass("mb-2");
  });
});
