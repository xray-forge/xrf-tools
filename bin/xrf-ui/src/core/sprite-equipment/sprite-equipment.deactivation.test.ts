import { beforeEach, describe, expect, it } from "@jest/globals";

import { SpriteEquipmentService } from "@/core/sprite-equipment/sprite-equipment.service";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

function closeCalls(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:sprite-equipment|close_sprite").length;
}

/**
 * These assert the container semantics the release hook depends on, not just that the hook exists.
 */
describe("SpriteEquipmentService deactivation", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
  });

  it("does not release on deprovision alone, which strict mode reaches on every mount", async () => {
    const { container } = mockInjectedService(SpriteEquipmentService);

    await container.provision();

    container.get(SpriteEquipmentService);
    container.deprovision();

    // The strict mode remount cancels the pending `unbindAll`, so this is the whole teardown it sees.
    // Releasing here would close a project the user is still looking at.
    expect(closeCalls()).toBe(0);
  });

  it("releases once the container is actually unbound", async () => {
    const { container } = mockInjectedService(SpriteEquipmentService);

    await container.provision();

    container.get(SpriteEquipmentService);

    container.deprovision();
    container.unbindAll();

    expect(closeCalls()).toBe(1);
  });

  it("survives a strict mode style remount without releasing", async () => {
    const { container } = mockInjectedService(SpriteEquipmentService);

    await container.provision();

    container.get(SpriteEquipmentService);

    // Mount, throwaway unmount, remount - `unbindAll` never runs because the provider cancels it.
    container.deprovision();

    await container.provision();

    container.get(SpriteEquipmentService);

    expect(closeCalls()).toBe(0);

    // Leaving for real still releases.
    container.deprovision();
    container.unbindAll();

    expect(closeCalls()).toBe(1);
  });
});
