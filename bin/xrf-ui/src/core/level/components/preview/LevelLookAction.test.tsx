import { afterEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { LevelLookAction } from "@/core/level/components/preview/LevelLookAction";
import { ELevelLookSource } from "@/core/level/lib/look";
import { LevelLookService } from "@/core/level/services";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderAction(): { look: LevelLookService } & ReturnType<typeof renderWithProviders> {
  const container: Container = mockContainer([LevelLookService]);

  return { ...renderWithProviders(<LevelLookAction />, { container }), look: container.get(LevelLookService) };
}

afterEach(() => {
  window.localStorage.clear();
});

describe("LevelLookAction", () => {
  it("draws with the settings' look, then saves the look as a preset it lists and can delete", async () => {
    const { getByRole, findByRole, look } = renderAction();

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await userEvent.click(getByRole("option", { name: "Settings" }));

    expect(look.choice.source).toBe(ELevelLookSource.SETTINGS);

    await userEvent.type(getByRole("textbox", { name: "Preset name" }), "dusk");
    await userEvent.click(getByRole("button", { name: "Save" }));

    expect(look.choice.preset).toBe("dusk");
    expect(getByRole("option", { name: "dusk" })).toHaveAttribute("aria-selected", "true");

    await userEvent.click(getByRole("button", { name: 'Delete "dusk"' }));

    expect(look.choice.presets).toEqual([]);
  });

  it("offers Anomaly's and OpenXRay's own looks whatever game is open", async () => {
    const { getByRole, findByRole, look } = renderAction();

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await userEvent.click(getByRole("option", { name: "Anomaly" }));

    expect(look.choice.source).toBe(ELevelLookSource.ANOMALY);
    expect(look.look.lightScales.sun).toBe(2);
    expect(look.look.exposure.middleGray).toBe(1.5);

    await userEvent.click(getByRole("option", { name: "OpenXRay" }));

    expect(look.look.lightScales.sun).toBe(1);
    expect(look.look.exposure.middleGray).toBe(1);
  });
});
