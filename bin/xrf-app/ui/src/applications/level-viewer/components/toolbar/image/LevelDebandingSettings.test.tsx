import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderDebandingMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelDebandingSettings } from "@/applications/level-viewer/components/toolbar/image/LevelDebandingSettings";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_RENDER_DEBANDING_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelDebandingSettings", () => {
  it("debands the sky on asking, offering its quality and radius only then", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ debanding: DEFAULT_RENDER_DEBANDING_SETTINGS });

    const { getByRole, queryByRole, view } = renderLevelView(<LevelDebandingSettings />, descriptor);

    expect(queryByRole("slider", { name: "Debanding radius" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Debanding" }));

    expect(view.features.debanding).toEqual({ mode: ERenderDebandingMode.ENHANCED });
    expect(getByRole("slider", { name: "Debanding radius" })).toHaveAttribute("aria-valuetext", "48 px");

    expect(getByRole("slider", { name: "Debanding passes" })).toHaveAttribute("aria-valuetext", "2 times");

    getByRole("slider", { name: "Debanding passes" }).focus();
    await userEvent.keyboard("{ArrowRight}");

    expect(view.features.debanding).toEqual({ mode: ERenderDebandingMode.ENHANCED, passes: 3 });

    await userEvent.click(getByRole("button", { name: "Reset the debanding to the preset" }));

    expect(view.features.debanding).toEqual({});
  });
});
