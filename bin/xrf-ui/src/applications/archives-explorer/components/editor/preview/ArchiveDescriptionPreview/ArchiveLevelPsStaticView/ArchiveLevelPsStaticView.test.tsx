import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveLevelPsStaticDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelPsStaticDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelPsStaticView } from "./ArchiveLevelPsStaticView";

function renderView(
  description: ArchiveLevelPsStaticDescription = mockArchiveLevelPsStaticDescription()
): RenderResult {
  return renderWithProviders(<ArchiveLevelPsStaticView description={description} />);
}

describe("ArchiveLevelPsStaticView", () => {
  it("lists what is planted rather than where each copy of it stands", () => {
    // A level plants a handful of effects a hundred times over, so the roll call is the effects.
    const { getByText } = renderView();

    expect(getByText("129")).toBeTruthy();
    expect(getByText("Effects (3)")).toBeTruthy();
    expect(getByText("65 placements")).toBeTruthy();
  });

  it("names an effect as the particles.xr definition it is, not as a file", () => {
    const { getByText } = renderView();

    expect(getByText("industrial\\steam_01")).toBeTruthy();
    expect(getByText("industrial\\steam_01").closest("button")).toBeNull();
    expect(getByText(/names a definition inside particles.xr/)).toBeTruthy();
  });

  it("says which placements a single-player session never plays, per effect and over the file", () => {
    const { getByText } = renderView();

    expect(getByText(/12 only some multiplayer modes load/)).toBeTruthy();
    expect(getByText("12 of them only some multiplayer modes load")).toBeTruthy();
  });

  it("says plainly when every placement plays, rather than reporting a restriction of none", () => {
    const { getByText } = renderView(
      mockArchiveLevelPsStaticDescription({
        restricted: 0,
        effects: [{ name: "weather\\rain_splash", placements: 1, restricted: 0 }],
      })
    );

    expect(getByText("Every one of them plays in a single-player session")).toBeTruthy();
    expect(getByText("1 placement")).toBeTruthy();
  });
});
