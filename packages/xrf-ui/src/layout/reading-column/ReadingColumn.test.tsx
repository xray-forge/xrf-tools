import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { ReadingColumn } from "./ReadingColumn";

describe("ReadingColumn", () => {
  it("centres its column at the reading width", () => {
    const { getByTestId } = renderWithTheme(
      <ReadingColumn data-testid={"description"}>
        <div>rows</div>
      </ReadingColumn>
    );

    const column: Element = getByTestId("description").firstElementChild as Element;

    expect(column).toHaveClass("mx-auto");
    expect(column).toHaveClass("max-w-reading");
    expect(column).toHaveTextContent("rows");
  });

  it("scrolls from the top, because what it holds is read down", () => {
    const { getByTestId } = renderWithTheme(
      <ReadingColumn data-testid={"description"} id={"reading"} className={"pt-4"}>
        <div>rows</div>
      </ReadingColumn>
    );

    const pane: Element = getByTestId("description");

    expect(pane).toHaveClass("overflow-y-auto");
    expect(pane).toHaveClass("pt-4");
    expect(pane).toHaveAttribute("id", "reading");
    expect(pane).not.toHaveClass("justify-center");
  });
});
