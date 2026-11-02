import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelParticlesAction } from "@/core/level/components/preview/toolbar/contents/LevelParticlesAction";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelParticlesAction", () => {
  it("says what plays and turns the particles, the campfires and the ambient effects over by their own checkboxes", async () => {
    const { container, view } = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelParticlesAction />, { container });

    expect(getByRole("button", { name: "Particles" })).toHaveAccessibleDescription("Particles playing, campfires lit");

    await userEvent.click(getByRole("button", { name: "Particles" }));
    await findByRole("dialog", { name: "Particles" });
    await userEvent.click(getByRole("checkbox", { name: "Particles" }));
    await userEvent.click(getByRole("checkbox", { name: "Campfires lit" }));
    await userEvent.click(getByRole("checkbox", { name: "Ambient effects" }));

    expect(view.options).toEqual({
      ...DEFAULT_LEVEL_VIEW_OPTIONS,
      isAmbientEffectPlayed: !DEFAULT_LEVEL_VIEW_OPTIONS.isAmbientEffectPlayed,
      isCampfireLit: !DEFAULT_LEVEL_VIEW_OPTIONS.isCampfireLit,
      isParticled: !DEFAULT_LEVEL_VIEW_OPTIONS.isParticled,
    });
  });

  it("says when the particles are off", () => {
    const { container, view } = mockLevelViewContainer();

    view.setOption("isParticled", false);

    const { getByRole } = renderWithProviders(<LevelParticlesAction />, { container });

    expect(getByRole("button", { name: "Particles" })).toHaveAccessibleDescription("Particles off");
  });
});
