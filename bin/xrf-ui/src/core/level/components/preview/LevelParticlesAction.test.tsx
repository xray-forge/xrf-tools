import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelParticlesAction } from "@/core/level/components/preview/LevelParticlesAction";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { renderWithProviders } from "@/fixtures/utils/render";

type TToggle = (option: keyof ILevelViewOptions) => void;

describe("LevelParticlesAction", () => {
  it("says what plays and turns the particles, the campfires and the ambient effects over by their own checkboxes", async () => {
    const onToggle = jest.fn<TToggle>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelParticlesAction options={DEFAULT_LEVEL_VIEW_OPTIONS} onToggle={onToggle} />
    );

    expect(getByRole("button", { name: "Particles" })).toHaveAccessibleDescription("Particles playing, campfires lit");

    await userEvent.click(getByRole("button", { name: "Particles" }));
    await findByRole("dialog", { name: "Particles" });
    await userEvent.click(getByRole("checkbox", { name: "Particles" }));
    await userEvent.click(getByRole("checkbox", { name: "Campfires lit" }));
    await userEvent.click(getByRole("checkbox", { name: "Ambient effects" }));

    expect(onToggle.mock.calls).toEqual([["isParticled"], ["isCampfireLit"], ["isAmbientPlayed"]]);
  });

  it("says when the particles are off", () => {
    const { getByRole } = renderWithProviders(
      <LevelParticlesAction options={{ ...DEFAULT_LEVEL_VIEW_OPTIONS, isParticled: false }} onToggle={() => {}} />
    );

    expect(getByRole("button", { name: "Particles" })).toHaveAccessibleDescription("Particles off");
  });
});
