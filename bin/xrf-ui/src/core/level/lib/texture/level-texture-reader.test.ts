import { describe, expect, it } from "@jest/globals";

import { createRoots } from "@/core/assets/lib";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import { LevelTextureReader } from "@/core/level/lib/texture/level-texture-reader";
import { MOCK_TRANSPORT_ENDPOINT, mockFetch } from "@/fixtures/mocks/bulk.mocks";
import { mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";

const ROOTS: XrayRoots = createRoots(["C:/game/db"]);

function request(reference: string): ISectorTextureRequest {
  return { reference };
}

describe("LevelTextureReader", () => {
  it("says where the file a reference resolved to is fetched, and where its picture is, without fetching either", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();
    const reference = mockLevelTextureReference("stone");

    reader.open(ROOTS, [reference]);

    const delivery: ILevelTextureDelivery = await reader.read(request("stone"));
    const headers: Record<string, string> = {
      Authorization: `Bearer ${MOCK_TRANSPORT_ENDPOINT.token}`,
      "Content-Type": "application/json",
    };
    const args = { roots: ROOTS, logicalPath: reference.logicalPath };
    const body: string = JSON.stringify(args);

    function batch(route: string): { call: string; url: string } {
      return { call: JSON.stringify({ args, route }), url: `${MOCK_TRANSPORT_ENDPOINT.origin}/batch` };
    }

    expect(delivery).toEqual({
      reason: null,
      reference: "stone",
      requests: {
        file: {
          batch: batch("assets/read_asset"),
          body,
          headers,
          url: `${MOCK_TRANSPORT_ENDPOINT.origin}/assets/read_asset`,
        },
        picture: {
          batch: batch("textures/read_texture"),
          body,
          headers,
          url: `${MOCK_TRANSPORT_ENDPOINT.origin}/textures/read_texture`,
        },
      },
    });
    // The renderer fetches them: no byte of a level's textures crosses the page.
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it("says why there is no file where the roots answer to nothing", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, []);

    const delivery: ILevelTextureDelivery = await reader.read(request("stone"));

    expect(delivery.requests).toBeNull();
    expect(delivery.reason).toContain("Nothing in the mounted roots answers to");
  });

  it("answers for references added beside the open", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, []);
    reader.add([mockLevelTextureReference("grass")]);

    expect((await reader.read(request("grass"))).requests).not.toBeNull();
  });

  it("finds nothing once the level has closed", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, [mockLevelTextureReference("stone")]);
    reader.close();

    expect((await reader.read(request("stone"))).reason).toContain("Nothing in the mounted roots answers to");
  });
});
