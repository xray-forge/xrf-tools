import { describe, expect, it } from "@jest/globals";

import {
  isLevelCubeSupersamplingDrawn,
  toDefaultLevelCubeParameters,
  toLevelCubeSupersampling,
  toRenderCubeCapture,
} from "@/applications/level-viewer/lib/cube/level-cube-capture";
import { DEFAULT_LEVEL_CUBE_EXPORT, toCubeExportRequest } from "@/applications/level-viewer/lib/cube/level-cube-export";
import { ECubeExportFormat } from "@/core/ipc/types/xrf-app";
import { ETextureEncodingFormat, ETextureEncodingQuality } from "@/core/ipc/types/xrf-texture";

describe("level cubemap capture", () => {
  it("draws faces twice over but the largest once", () => {
    expect(toLevelCubeSupersampling(512)).toBe(2);
    expect(toLevelCubeSupersampling(1024)).toBe(2);
    expect(toLevelCubeSupersampling(2048)).toBe(1);
    expect(isLevelCubeSupersamplingDrawn(2048, 4)).toBe(true);
    expect(isLevelCubeSupersamplingDrawn(2048, 8)).toBe(false);
  });

  it("asks the renderer for the faces the parameters say", () => {
    expect(toRenderCubeCapture(toDefaultLevelCubeParameters())).toEqual({
      size: 512,
      supersampling: 2,
      settleFrames: 16,
    });
  });

  it("exports a BC7 cube at most the capture's face size, six files as their kind alone", () => {
    expect(toCubeExportRequest(DEFAULT_LEVEL_CUBE_EXPORT, "cross.png", "cube.dds", 512)).toEqual({
      path: "cross.png",
      target: "cube.dds",
      format: {
        kind: ECubeExportFormat.DDS,
        format: ETextureEncodingFormat.BC7,
        quality: ETextureEncodingQuality.SLOW,
        isMipmapped: true,
      },
      size: 512,
    });
    expect(
      toCubeExportRequest({ ...DEFAULT_LEVEL_CUBE_EXPORT, kind: ECubeExportFormat.TGA, size: 64 }, "x.png", "out", 512)
    ).toMatchObject({ format: { kind: ECubeExportFormat.TGA }, size: 64 });
  });
});
