import { describe, expect, it } from "@jest/globals";

import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveDescriptionLayout } from "./ArchiveDescriptionLayout";

describe("ArchiveDescriptionLayout", () => {
  it("centres its reading column, the way the panes beside it are centred", () => {
    const { getByTestId } = renderWithProviders(
      <ArchiveDescriptionLayout data-testid={"description"}>
        <div>rows</div>
      </ArchiveDescriptionLayout>
    );

    const column: Element = getByTestId("description").firstElementChild as Element;

    expect(column).toHaveClass("mx-auto");
    expect(column).toHaveClass("max-w-reading");
  });

  it("stays anchored to the top, because a description scrolls", () => {
    const { getByTestId } = renderWithProviders(
      <ArchiveDescriptionLayout data-testid={"description"}>
        <div>rows</div>
      </ArchiveDescriptionLayout>
    );

    const pane: Element = getByTestId("description");

    expect(pane).toHaveClass("overflow-y-auto");
    expect(pane).not.toHaveClass("justify-center");
  });
});
