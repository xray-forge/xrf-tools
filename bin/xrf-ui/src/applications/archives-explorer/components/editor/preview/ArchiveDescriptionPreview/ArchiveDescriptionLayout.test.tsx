import { describe, expect, it } from "@jest/globals";

import { LAYOUT } from "@/core/theme/tokens";
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
    const style: CSSStyleDeclaration = getComputedStyle(column);

    expect(style.marginLeft).toBe("auto");
    expect(style.marginRight).toBe("auto");
    expect(style.maxWidth).toBe(`${LAYOUT.readingColumnWidth}px`);
  });

  it("stays anchored to the top, because a description scrolls", () => {
    const { getByTestId } = renderWithProviders(
      <ArchiveDescriptionLayout data-testid={"description"}>
        <div>rows</div>
      </ArchiveDescriptionLayout>
    );

    const style: CSSStyleDeclaration = getComputedStyle(getByTestId("description"));

    expect(style.overflowY).toBe("auto");
    expect(style.justifyContent).not.toBe("center");
  });
});
