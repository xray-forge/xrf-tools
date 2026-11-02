import { describe, expect, it } from "@jest/globals";
import { ERenderDrawableKind } from "@xrf/ipc/types/xrf-renderer";

import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelRendererDrawablesSection } from "@/tools/level-viewer/components/panels/LevelRendererPanel/LevelRendererDrawablesSection";

describe("LevelRendererDrawablesSection", () => {
  it("shows a section a kind, titled by the kind and counted in what it draws", () => {
    const { getByText } = renderWithProviders(
      <LevelRendererDrawablesSection drawables={[{ capacity: 4096, drawn: 1200, kind: ERenderDrawableKind.GRASS }]} />
    );

    expect(getByText("Grass")).toBeInTheDocument();
    expect(getByText(/^1,?200 tufts$/)).toBeInTheDocument();
  });

  it("says the lists ran full where the cull kept more than they hold", () => {
    const { getByText } = renderWithProviders(
      <LevelRendererDrawablesSection drawables={[{ capacity: 100, drawn: 150, kind: ERenderDrawableKind.GRASS }]} />
    );

    expect(getByText(/the lists full/)).toBeInTheDocument();
  });

  it("shows nothing without a kind drawn", () => {
    const { queryByText } = renderWithProviders(<LevelRendererDrawablesSection drawables={[]} />);

    expect(queryByText("Grass")).not.toBeInTheDocument();
  });
});
