import { describe, expect, it } from "@jest/globals";

import { ELevelRenderRequest, listRenderTransfers } from "@/core/level/lib/render/level-render-messages";
import { ILevelSectorDelivery, ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { mockSectorDescription } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

function sector(sector: number, bytes: number): ILevelSectorDelivery {
  return { buffer: new ArrayBuffer(bytes), description: mockSectorDescription(new MockVisualBuffer()), sector };
}

function texture(reference: string, bytes: number): ILevelTextureDelivery {
  return {
    bytes: new ArrayBuffer(bytes),
    isAlphaRead: false,
    isDecoded: false,
    isMipped: true,
    reason: null,
    reference,
  };
}

/**
 * A sector is a megabyte or so of pack and a texture is its file. Copying either for every message is the one
 * thing that would make drawing on another thread slower than not bothering.
 */
describe("listRenderTransfers", () => {
  it("moves a sector's pack rather than copying it", () => {
    const first: ILevelSectorDelivery = sector(0, 16);
    const second: ILevelSectorDelivery = sector(1, 32);

    const transfers = listRenderTransfers({
      change: { delivered: [first, second], released: [] },
      kind: ELevelRenderRequest.DELIVER,
    });

    expect(transfers).toEqual([first.buffer, second.buffer]);
  });

  it("moves a texture's file rather than copying it", () => {
    const file: ILevelTextureDelivery = texture("stone", 64);

    const transfers = listRenderTransfers({
      change: { delivered: [file], retained: null },
      kind: ELevelRenderRequest.SUPPLY,
    });

    expect(transfers).toEqual([file.bytes]);
  });

  it("moves nothing for a message that carries nothing", () => {
    expect(listRenderTransfers({ id: 1, kind: ELevelRenderRequest.MEASURE })).toEqual([]);
    expect(listRenderTransfers({ kind: ELevelRenderRequest.OPEN, level: null })).toEqual([]);
  });

  // A release names sectors by number and carries no pack, so there is nothing to move and nothing to lose.
  it("moves nothing for a change that only releases", () => {
    expect(
      listRenderTransfers({ change: { delivered: [], released: [4, 5] }, kind: ELevelRenderRequest.DELIVER })
    ).toEqual([]);
  });
});
