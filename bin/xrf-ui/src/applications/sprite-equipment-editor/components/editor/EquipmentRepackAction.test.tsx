import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Injectable } from "@wirestate/core";

import { EquipmentRepackAction } from "@/applications/sprite-equipment-editor/components/editor/EquipmentRepackAction";
import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { AssetService } from "@/core/assets/services";
import { EquipmentSpriteMetadata } from "@/core/ipc/types/xrf-app";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockEquipmentSpriteMetadata, mockEquipmentSpriteOpen } from "@/fixtures/mocks/sprite.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

/** What the service under test starts holding, which each case varies before rendering. */
const seed: { repackSourcePath: Nullable<string>; metadata: EquipmentSpriteMetadata } = {
  repackSourcePath: null,
  metadata: mockEquipmentSpriteMetadata(),
};

/** The instance the container built for the current render, so a test can watch what it is asked to do. */
let rendered: Nullable<SpriteEquipmentEditorService> = null;

function captureRendered(service: SpriteEquipmentEditorService): void {
  rendered = service;
}

/**
 * A service that starts in the state under test.
 *
 * Subclassed rather than bound as a prepared instance because the container only provisions objects it
 * constructed itself, and `useInjection` goes through provisioning.
 */
@Injectable()
class TestSpriteEquipmentEditorService extends SpriteEquipmentEditorService {
  /** The fixture already contains the restored sprite and does not read a backend project. */
  public override async onProvision(): Promise<void> {}

  public constructor() {
    super();

    this.spriteImage = this.spriteImage.asReady({
      sessionId: "fixture-session",
      metadata: seed.metadata,
      blob: new Blob(),
      image: new Image(),
    });
    this.repackSourcePath = seed.repackSourcePath;

    captureRendered(this);
  }
}

function renderAction(
  repackSourcePath: Nullable<string>,
  metadata: Partial<EquipmentSpriteMetadata> = {}
): RenderResult {
  seed.repackSourcePath = repackSourcePath;
  seed.metadata = mockEquipmentSpriteMetadata(metadata);

  return renderWithProviders(<EquipmentRepackAction />, {
    bindings: [
      AssetService,
      SpriteEquipmentPackerService,
      { token: SpriteEquipmentEditorService, type: "Instance", value: TestSpriteEquipmentEditorService },
    ],
  });
}

describe("EquipmentRepackAction", () => {
  it("withholds repacking when there is nothing to rebuild from", () => {
    const { getByRole, getByTitle } = renderAction(null);

    // Previously the command was offered, then failed after the click, in the console.
    expect(getByRole("button", { name: /Repack/ })).toBeDisabled();
    // The reason moved into the tooltip when the command became a toolbar button, so it still has to
    // be reachable rather than leaving a dead control with no explanation.
    expect(getByTitle("No unpacked icons beside the sprite")).toBeInTheDocument();
  });

  it("withholds repacking a sheet that was not opened from files", () => {
    // A tree open resolves its configuration to an entry point rather than a file, and an archived sheet has no path
    // at all. Neither can be handed to the packer, and saying so is a different fix from unpacking the icons first.
    const { getByRole, getByTitle } = renderAction("C:\\game\\equipment", {
      open: mockEquipmentSpriteOpen({
        sheet: { kind: "asset", reference: "ui\\ui_icon_equipment" },
        config: { kind: "asset", logicalPath: "configs\\system.ltx" },
      }),
    });

    expect(getByRole("button", { name: /Repack/ })).toBeDisabled();
    expect(getByTitle("This sheet was not opened from files a repack can write")).toBeInTheDocument();
  });

  it("names both paths before overwriting anything", async () => {
    const { getByRole, findByText } = renderAction("C:\\game\\equipment");

    await userEvent.click(getByRole("button", { name: /Repack/ }));

    // The dialog has to say what is read and what is destroyed, not just that something will happen.
    expect(await findByText("C:\\game\\equipment")).toBeInTheDocument();
    expect(await findByText("C:\\game\\equipment.dds")).toBeInTheDocument();
    expect(getByRole("button", { name: "Repack" })).toBeInTheDocument();
  });

  it("does not repack when the confirmation is dismissed", async () => {
    const { getByRole, queryByRole } = renderAction("C:\\game\\equipment");

    await userEvent.click(getByRole("button", { name: /Repack/ }));
    await userEvent.click(getByRole("button", { name: "Cancel" }));

    // The dialog leaves on a transition, so its absence has to be waited for rather than asserted
    // straight after the click.
    await waitFor(() => expect(queryByRole("button", { name: "Repack" })).not.toBeInTheDocument());

    // Asserted through the service rather than a spy: `@BoundAction()` makes the method non-writable,
    // and untouched state is the stronger claim anyway. A repack that ran against the mocked backend
    // would have left either a timestamp or an error behind.
    expect((rendered as SpriteEquipmentEditorService).repackedAt).toBeNull();
    expect((rendered as SpriteEquipmentEditorService).spriteImage.error).toBeNull();
    expect((rendered as SpriteEquipmentEditorService).spriteImage.isLoading).toBe(false);
  });
});
