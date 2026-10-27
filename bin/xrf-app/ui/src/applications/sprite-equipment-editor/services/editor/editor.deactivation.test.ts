import { beforeEach, describe, expect, it } from "@jest/globals";
import { ESessionsCommand } from "@xrf/ipc/commands/sessions";
import { ESpriteEquipmentCommand } from "@xrf/ipc/commands/sprite-equipment";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";

import { mockShownEquipmentSprite } from "@/applications/sprite-equipment-editor/fixtures/editor.mocks";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockInjectedService } from "@/fixtures/utils/container";

import { SpriteEquipmentEditorService } from "./editor.service";

/** How many times the tool's scope was released, which closes the sprite with whatever else the tool held. */
function closeCalls(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === ESessionsCommand.RELEASE_SCOPE).length;
}

/** Whether the service closed its sprite itself, which leaving the tool leaves to the scope. */
function hasClosedItself(): boolean {
  return mockInvoke.mock.calls.some(([command]) => command === ESpriteEquipmentCommand.CLOSE_SPRITE);
}

/** Puts a sprite on screen the way a restore does, so the service owns the opening its teardown releases. */
function showSprite(service: SpriteEquipmentEditorService): void {
  mockShownEquipmentSprite(service);
}

/**
 * These assert the container semantics the release hook depends on, not just that the hook exists.
 */
describe("SpriteEquipmentEditorService deactivation", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("does not release on deprovision alone, which strict mode reaches on every mount", async () => {
    const { container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    await container.provision();

    showSprite(container.get(SpriteEquipmentEditorService));

    container.deprovision();

    // The strict mode remount cancels the pending `unbindAll`, so this is the whole teardown it sees.
    // Releasing here would close a project the user is still looking at.
    expect(closeCalls()).toBe(0);
  });

  it("releases once the container is actually unbound", async () => {
    const { container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    await container.provision();

    showSprite(container.get(SpriteEquipmentEditorService));

    container.deprovision();
    container.unbindAll();

    expect(closeCalls()).toBe(1);
    expect(hasClosedItself()).toBe(false);
  });

  it("survives a strict mode style remount without releasing", async () => {
    const { container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    await container.provision();

    showSprite(container.get(SpriteEquipmentEditorService));

    // Mount, throwaway unmount, remount - `unbindAll` never runs because the provider cancels it.
    container.deprovision();

    await container.provision();

    showSprite(container.get(SpriteEquipmentEditorService));

    expect(closeCalls()).toBe(0);

    // Leaving for real still releases.
    container.deprovision();
    container.unbindAll();

    expect(closeCalls()).toBe(1);
  });
});
