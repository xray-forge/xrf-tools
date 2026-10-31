import { describe, expect, it } from "@jest/globals";
import { ELevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";
import { RENDER_HELPER_ICONS, RenderHelperIconEntry } from "@xrf/ipc/types/xrf-renderer";

import { LEVEL_KIND_ICONS, LevelKindIcon } from "@/core/level/components/objects/LevelKindIcon";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelKindIcon", () => {
  // The renderer rasterises its atlas from the Material icons its table names; a row showing another picture than the
  // billboard drawn for the kind would mislead.
  it("shows the Material icon the renderer's atlas draws for each helper icon", () => {
    const { getByTestId } = renderWithProviders(
      <>
        {RENDER_HELPER_ICONS.map((entry: RenderHelperIconEntry) => {
          const Icon = LEVEL_KIND_ICONS[entry.icon];

          return <Icon key={entry.icon} />;
        })}
      </>
    );

    // A Material icon names itself in its test id.
    for (const entry of RENDER_HELPER_ICONS) {
      expect(getByTestId(`${entry.material}Icon`)).toBeInTheDocument();
    }

    expect(Object.keys(LEVEL_KIND_ICONS)).toHaveLength(RENDER_HELPER_ICONS.length);
  });

  it("draws a point kind's icon and nothing for a kind drawing none", () => {
    const { queryAllByTestId } = renderWithProviders(
      <>
        <LevelKindIcon kind={ELevelObjectKind.LIGHTS} />
        <LevelKindIcon kind={ELevelObjectKind.RESTRICTORS} />
      </>
    );

    expect(queryAllByTestId("level-kind-icon")).toHaveLength(1);
  });
});
