import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Injectable } from "@wirestate/core";
import { Nullable } from "@xrf/types";

import { AssetService } from "@/core/assets/services";
import { ENGINE_GRID_SQUARE } from "@/core/sprite-equipment/lib";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import { SpriteEquipmentEditorService } from "@/tools/sprite-equipment-editor/services/sprite-equipment-editor";

import { EquipmentGridOptions } from "./EquipmentGridOptions";

/** The instance the container built for the current render, so a test can set and read its grid. */
let rendered: Nullable<SpriteEquipmentEditorService> = null;

function captureRendered(service: SpriteEquipmentEditorService): void {
  rendered = service;
}

/** A service that reads no backend project, which the grid needs none of. */
@Injectable()
class TestSpriteEquipmentEditorService extends SpriteEquipmentEditorService {
  public override async onProvision(): Promise<void> {}

  public constructor() {
    super();

    captureRendered(this);
  }
}

beforeEach(() => {
  clearLocalStorage();
});

describe("EquipmentGridOptions", () => {
  it("sets the cells back to the engine's size from the heading, only while they are off it", async () => {
    const { getByRole, queryByRole } = renderWithProviders(<EquipmentGridOptions />, {
      bindings: [
        AssetService,
        SpriteEquipmentPackerService,
        { token: SpriteEquipmentEditorService, type: "Instance", value: TestSpriteEquipmentEditorService },
      ],
    });
    const reset: { name: string } = { name: `Grid: Reset to the engine's ${ENGINE_GRID_SQUARE}px` };
    const service: SpriteEquipmentEditorService = rendered as SpriteEquipmentEditorService;

    await userEvent.click(getByRole("button", { name: "Grid" }));

    expect(queryByRole("button", reset)).toBeNull();

    await userEvent.click(getByRole("button", { name: "Increase grid size" }));

    expect(getByRole("group", { name: "Cell size" })).toHaveTextContent(`${ENGINE_GRID_SQUARE + 5}px`);

    await userEvent.click(getByRole("button", reset));

    expect(service.gridSize).toBe(ENGINE_GRID_SQUARE);
    expect(queryByRole("button", reset)).toBeNull();

    act(() => service.setGridVisibility(false));

    expect(getByRole("checkbox", { name: "Show cells" })).not.toBeChecked();
  });
});
