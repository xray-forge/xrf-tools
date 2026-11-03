import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelSoundAction } from "@/core/level/components/preview/toolbar/sound/LevelSoundAction";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelSoundAction", () => {
  it("says the sound is heard and turns it and each category over by their own checkboxes", async () => {
    const { container, view } = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelSoundAction />, { container });

    expect(getByRole("button", { name: "Sound" })).toHaveAccessibleDescription(
      "Sound heard while this viewport listens"
    );

    await userEvent.click(getByRole("button", { name: "Sound" }));
    await findByRole("dialog", { name: "Sound" });
    await userEvent.click(getByRole("checkbox", { name: "Music" }));
    await userEvent.click(getByRole("checkbox", { name: "Ambience" }));
    await userEvent.click(getByRole("checkbox", { name: "World" }));
    await userEvent.click(getByRole("checkbox", { name: "Sound" }));

    expect(view.options).toEqual({
      ...DEFAULT_LEVEL_VIEW_OPTIONS,
      isAmbienceHeard: false,
      isMusicHeard: false,
      isSoundHeard: false,
      isWorldHeard: false,
    });
  });

  it("says when the sound is off", () => {
    const { container, view } = mockLevelViewContainer();

    view.setOption("isSoundHeard", false);

    const { getByRole } = renderWithProviders(<LevelSoundAction />, { container });

    expect(getByRole("button", { name: "Sound" })).toHaveAccessibleDescription("Sound off");
  });
});
