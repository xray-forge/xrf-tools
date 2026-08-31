import { describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";

import { AssetService } from "@/core/assets/services";
import { SpriteEquipmentService } from "@/core/sprite-equipment/sprite-equipment.service";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

const RESPONSE = {
  name: "equipment.dds",
  path: "C:\\game\\equipment.dds",
  systemLtxPath: "C:\\game\\system.ltx",
  equipmentDescriptors: [],
};

/**
 * The sprite is fetched through `convertFileSrc` and `fetch`, neither of which jsdom provides, so the
 * blob arrives from a stub. What matters here is the url lifetime, not what the bytes decode to.
 *
 * @returns The service, its asset service, and their provisioned container.
 */
function createService(): { service: SpriteEquipmentService; assets: AssetService; container: Container } {
  const { service, container } = mockInjectedService(SpriteEquipmentService);

  global.fetch = jest.fn(async () => ({ blob: async () => new Blob() })) as unknown as typeof fetch;

  // jsdom never loads an image, so `onload` would never fire and the decode would hang. Resolving on
  // assignment keeps the url visible as `src`, which is what these assertions are about.
  global.Image = class {
    public onload: Nullable<() => void> = null;
    public onerror: Nullable<() => void> = null;

    private source: string = "";

    public get src(): string {
      return this.source;
    }

    public set src(value: string) {
      this.source = value;
      queueMicrotask(() => this.onload?.());
    }
  } as unknown as typeof Image;

  return {
    assets: container.get(AssetService),
    service,
    container,
  };
}

describe("SpriteEquipmentService object urls", () => {
  it("holds exactly one url no matter how often the sprite is reloaded", async () => {
    setMockInvokeResponses({ ["plugin:sprite-equipment|reopen_sprite"]: RESPONSE });

    const { service, assets } = createService();

    await service.reopenEquipmentProject();
    await service.reopenEquipmentProject();
    await service.reopenEquipmentProject();

    // Each reload swaps the url under one key. Growing here is the leak that had `blobToImage` give up
    // and comment its revoke out.
    expect(assets.heldCount).toBe(1);
  });

  it("keeps the url the reload just produced rather than revoking it", async () => {
    setMockInvokeResponses({ ["plugin:sprite-equipment|reopen_sprite"]: RESPONSE });

    const { service, assets } = createService();

    const revoked: Array<string> = [];

    jest.spyOn(URL, "revokeObjectURL").mockImplementation((url: string) => {
      revoked.push(url);
    });

    await service.reopenEquipmentProject();

    const current: string | undefined = service.spriteImage.value?.image.src;

    await service.reopenEquipmentProject();

    // Releasing after the swap would revoke the replacement, blanking the viewer.
    expect(current).toBeDefined();
    expect(revoked).toEqual([current]);
    expect(revoked).not.toContain(service.spriteImage.value?.image.src);
    expect(assets.heldCount).toBe(1);
  });

  it("releases the sprite url when the editor is navigated away from", async () => {
    setMockInvokeResponses({ ["plugin:sprite-equipment|reopen_sprite"]: RESPONSE });

    const { service, assets, container } = createService();

    await service.reopenEquipmentProject();
    expect(assets.heldCount).toBe(1);

    container.deprovision();
    container.unbindAll();

    expect(assets.heldCount).toBe(0);
  });
});
