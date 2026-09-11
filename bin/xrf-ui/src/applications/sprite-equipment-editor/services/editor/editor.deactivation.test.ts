import { beforeEach, describe, expect, it } from "@jest/globals";

import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { SpriteEquipmentEditorService } from "./editor.service";

function closeCalls(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:sprite-equipment|close_sprite").length;
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

    container.get(SpriteEquipmentEditorService).spriteImage = container
      .get(SpriteEquipmentEditorService)
      .spriteImage.asReady({
        sessionId: "fixture-session",
        isDltx: false,
        ltxPath: "system.ltx",
        descriptors: [],
        path: "equipment.dds",
        name: "equipment.png",
        blob: new Blob(),
        image: new Image(),
      });

    container.deprovision();

    // The strict mode remount cancels the pending `unbindAll`, so this is the whole teardown it sees.
    // Releasing here would close a project the user is still looking at.
    expect(closeCalls()).toBe(0);
  });

  it("releases once the container is actually unbound", async () => {
    const { container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    await container.provision();

    container.get(SpriteEquipmentEditorService).spriteImage = container
      .get(SpriteEquipmentEditorService)
      .spriteImage.asReady({
        sessionId: "fixture-session",
        isDltx: false,
        ltxPath: "system.ltx",
        descriptors: [],
        path: "equipment.dds",
        name: "equipment.png",
        blob: new Blob(),
        image: new Image(),
      });

    container.deprovision();
    container.unbindAll();

    expect(closeCalls()).toBe(1);
  });

  it("survives a strict mode style remount without releasing", async () => {
    const { container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    await container.provision();

    container.get(SpriteEquipmentEditorService).spriteImage = container
      .get(SpriteEquipmentEditorService)
      .spriteImage.asReady({
        sessionId: "fixture-session",
        isDltx: false,
        ltxPath: "system.ltx",
        descriptors: [],
        path: "equipment.dds",
        name: "equipment.png",
        blob: new Blob(),
        image: new Image(),
      });

    // Mount, throwaway unmount, remount - `unbindAll` never runs because the provider cancels it.
    container.deprovision();

    await container.provision();

    container.get(SpriteEquipmentEditorService).spriteImage = container
      .get(SpriteEquipmentEditorService)
      .spriteImage.asReady({
        sessionId: "fixture-session",
        isDltx: false,
        ltxPath: "system.ltx",
        descriptors: [],
        path: "equipment.dds",
        name: "equipment.png",
        blob: new Blob(),
        image: new Image(),
      });

    expect(closeCalls()).toBe(0);

    // Leaving for real still releases.
    container.deprovision();
    container.unbindAll();

    expect(closeCalls()).toBe(1);
  });
});
