import { describe, expect, it } from "@jest/globals";

import {
  ELevelMapArea,
  ELevelMapLook,
  ILevelMapRect,
  ILevelMapVolume,
  isLevelMapSupersamplingDrawn,
  toDefaultLevelMapParameters,
  toLevelMapAreaRect,
  toLevelMapMetresPerPixel,
  toLevelMapSize,
  toLevelMapSupersampling,
  toLevelMapTileCount,
  toLevelMapVolume,
  toRenderMapCapture,
} from "@/tools/level-viewer/lib/map/level-map-capture";

const ZATON: ILevelMapRect = { minX: -600, minZ: -615, maxX: 600, maxZ: 585 };

const BOX: ILevelMapVolume = { rect: { minX: -700, minZ: -650, maxX: 700, maxZ: 640 }, bottom: -20, top: 80 };

describe("level map capture", () => {
  it("follows the area's shape on the shorter side", () => {
    expect(toLevelMapSize(ZATON, 2048)).toEqual([2048, 2048]);
    expect(toLevelMapSize({ minX: 0, minZ: 0, maxX: 300, maxZ: 200 }, 4096)).toEqual([4096, 2731]);
    expect(toLevelMapSize({ minX: 0, minZ: 0, maxX: 200, maxZ: 300 }, 4096)).toEqual([2731, 4096]);
  });

  it("measures the ground a pixel spans", () => {
    expect(toLevelMapMetresPerPixel(ZATON, 4096)).toBeCloseTo(0.293, 3);
  });

  it("draws twice over below the largest map", () => {
    expect(toLevelMapSupersampling(4096)).toBe(2);
    expect(toLevelMapSupersampling(8192)).toBe(1);
  });

  it("asks the renderer for the area at its size", () => {
    expect(
      toRenderMapCapture({
        area: ELevelMapArea.LEVEL_MAP,
        rect: ZATON,
        bottom: -20,
        top: 80,
        longSide: 1024,
        look: ELevelMapLook.MAP,
        isSpawned: false,
        isWatered: true,
        isShadowed: true,
        isGrassed: false,
        supersampling: 2,
        settleFrames: 16,
      })
    ).toEqual({
      minX: -600,
      minZ: -615,
      maxX: 600,
      maxZ: 585,
      bottom: -20,
      top: 80,
      width: 1024,
      height: 1024,
      supersampling: 2,
      settleFrames: 16,
      isGrassed: false,
    });
  });

  it("cuts a map into the renderer's tiles", () => {
    expect(toLevelMapTileCount(2048, 2048)).toBe(1);
    expect(toLevelMapTileCount(4096, 2731)).toBe(4);
  });

  it("draws four times over only what every GPU holds", () => {
    expect(isLevelMapSupersamplingDrawn(1024, 4)).toBe(true);
    expect(isLevelMapSupersamplingDrawn(2048, 4)).toBe(false);
    expect(isLevelMapSupersamplingDrawn(8192, 2)).toBe(true);
  });

  it("takes the area from where it comes, a custom one keeping what it has", () => {
    const custom: ILevelMapRect = { minX: 0, minZ: 0, maxX: 10, maxZ: 10 };

    expect(toLevelMapAreaRect(ELevelMapArea.LEVEL_MAP, ZATON, BOX, custom)).toBe(ZATON);
    expect(toLevelMapAreaRect(ELevelMapArea.LEVEL_MAP, null, BOX, custom)).toBe(BOX.rect);
    expect(toLevelMapAreaRect(ELevelMapArea.LEVEL_GEOMETRY, ZATON, BOX, custom)).toBe(BOX.rect);
    expect(toLevelMapAreaRect(ELevelMapArea.CUSTOM, ZATON, BOX, custom)).toBe(custom);
  });

  it("opens on the level's map, or its box where it lays none", () => {
    expect(toDefaultLevelMapParameters(ZATON, BOX)).toMatchObject({ area: ELevelMapArea.LEVEL_MAP, rect: ZATON });
    expect(toDefaultLevelMapParameters(null, BOX)).toMatchObject({
      area: ELevelMapArea.LEVEL_GEOMETRY,
      rect: BOX.rect,
      bottom: -20,
      top: 80,
    });
  });

  it("reads a level's box in engine space, z the other way", () => {
    expect(
      toLevelMapVolume({
        boundingBox: { min: { x: -1, y: -2, z: -3 }, max: { x: 4, y: 5, z: 6 } },
        boundingSphere: { center: { x: 0, y: 0, z: 0 }, radius: 1 },
      })
    ).toEqual({ rect: { minX: -1, minZ: -6, maxX: 4, maxZ: 3 }, bottom: -2, top: 5 });
  });
});
