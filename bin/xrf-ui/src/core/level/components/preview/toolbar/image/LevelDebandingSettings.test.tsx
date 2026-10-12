import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderDebandingMode, ERenderDebandingQuality } from "@/core/ipc/types/xrf-renderer";
import { LevelDebandingSettings } from "@/core/level/components/preview/toolbar/image/LevelDebandingSettings";
import { DEFAULT_RENDER_DEBANDING_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/fixtures/utils/level-container";

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

    await userEvent.click(getByRole("button", { name: "Ultra" }));

    expect(view.features.debanding).toEqual({
      mode: ERenderDebandingMode.ENHANCED,
      quality: ERenderDebandingQuality.ULTRA,
    });

    await userEvent.click(getByRole("button", { name: "Reset the debanding to the preset" }));

    expect(view.features.debanding).toEqual({});
  });
});
