import { beforeEach, describe, expect, it } from "@jest/globals";

import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockRestoredSession } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { SpriteEquipmentEditorService } from "./editor.service";

function closeCalls(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:sprite-equipment|close_sprite").length;
}

/** Puts a sprite on screen the way a restore does, so the service owns the opening its teardown releases. */
function showSprite(service: SpriteEquipmentEditorService): void {
  service.spriteImage = service.spriteImage.asReady(
    mockRestoredSession(service, {
      sessionId: "fixture-session",
      isDltx: false,
      ltxPath: "system.ltx",
      descriptors: [],
      path: "equipment.dds",
      name: "equipment.png",
      blob: new Blob(),
      image: new Image(),
    })
  );
}

/**
 * These assert the container semantics the release hook depends on, not just that the hook exists.
 */
describe("SpriteEquipmentEditorService deactivation", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
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
