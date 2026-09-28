import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRequest, TRendererRequest } from "#/contract/renderer-request";
import { ERendererResponse, TRendererResponse } from "#/contract/renderer-response";
import { IRendererSettings } from "#/contract/renderer-settings";
import { RendererDevice } from "#/device/renderer-device";
import { RendererDeviceFailure } from "#/device/renderer-device-failure";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";
import { RendererHost } from "#/host/renderer-host";
import { RendererScene } from "#/scene/renderer-scene";

const SETTINGS: IRendererSettings = {
  backdrop: null,
  debugView: ERendererDebugView.FINAL,
  features: RENDERER_PRESETS[ERendererPreset.BASE],
  hemiStrength: 1,
  isBumped: true,
  isLit: true,
  isSkyDrawn: false,
  isWireframe: false,
  pacing: DEFAULT_RENDER_FRAME_PACING,
  tonemapScale: 1,
};

function createHost(): [RendererHost, Array<TRendererResponse>] {
  const replies: Array<TRendererResponse> = [];
  const host: RendererHost = new RendererHost(
    (response: TRendererResponse) => {
      if (response.kind !== ERendererResponse.CURSOR) {
        replies.push(response);
      }
    },
    () => 0,
    () => {}
  );

  return [host, replies];
}

async function settle(): Promise<void> {
  for (let turn: number = 0; turn < 4; turn += 1) {
    await Promise.resolve();
  }
}

describe("RendererHost", () => {
  let error: jest.SpiedFunction<typeof console.error>;

  beforeEach(() => {
    error = jest.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("opens one device for as long as it lives, however often it is told to start", () => {
    const open: jest.SpiedFunction<typeof RendererDevice.open> = jest
      .spyOn(RendererDevice, "open")
      .mockReturnValue(new Promise<RendererDevice>(() => {}));
    const [host]: [RendererHost, Array<TRendererResponse>] = createHost();

    host.take({ kind: ERendererRequest.START, settings: SETTINGS });
    host.take({ kind: ERendererRequest.START, settings: SETTINGS });

    expect(open).toHaveBeenCalledTimes(1);
  });

  it("says why it cannot start, and takes nothing after: a consumer that wants to draw makes another", async () => {
    jest.spyOn(RendererDevice, "open").mockRejectedValue(new RendererDeviceFailure("WebGPU is unavailable."));

    const dispose: jest.SpiedFunction<() => void> = jest.spyOn(RendererScene.prototype, "dispose");
    const release: jest.SpiedFunction<(key: string) => void> = jest.spyOn(RendererScene.prototype, "releaseTexture");
    const [host, replies]: [RendererHost, Array<TRendererResponse>] = createHost();

    host.take({ kind: ERendererRequest.START, settings: SETTINGS });
    await settle();
    host.take({ key: "a", kind: ERendererRequest.RELEASE_TEXTURE });

    expect(replies).toEqual([{ kind: ERendererResponse.FAILED, reason: "WebGPU is unavailable." }]);
    expect(dispose).toHaveBeenCalledTimes(1);
    expect(release).not.toHaveBeenCalled();
  });

  // The page used to see only a generic worker error, while the worker ran on in a state nothing defined.
  it("fails for good on a request that throws, saying what was thrown", () => {
    jest.spyOn(RendererScene.prototype, "releaseTexture").mockImplementation(() => {
      throw new RangeError("out of range");
    });

    const [host, replies]: [RendererHost, Array<TRendererResponse>] = createHost();

    host.take({ key: "a", kind: ERendererRequest.RELEASE_TEXTURE });
    host.take({ key: "b", kind: ERendererRequest.RELEASE_TEXTURE });

    expect(replies).toEqual([
      { kind: ERendererResponse.FAILED, reason: "The renderer failed on a request: RangeError: out of range" },
    ]);
    expect(error).toHaveBeenCalledTimes(1);
  });

  it("applies a batch as one change, its end outside that change, and nothing after its end", () => {
    let depth: number = 0;
    const disposedAt: Array<number> = [];
    const released: Array<string> = [];
    const transact: (change: () => void) => void = RendererScene.prototype.transact;

    jest.spyOn(RendererScene.prototype, "transact").mockImplementation(function (this: RendererScene, change) {
      depth += 1;
      transact.call(this, change);
      depth -= 1;
    });
    jest.spyOn(RendererScene.prototype, "dispose").mockImplementation(() => {
      disposedAt.push(depth);
    });
    jest.spyOn(RendererScene.prototype, "releaseTexture").mockImplementation((key: string) => {
      released.push(`${key}:${depth}`);
    });

    const [host]: [RendererHost, Array<TRendererResponse>] = createHost();
    const requests: Array<TRendererRequest> = [
      { key: "a", kind: ERendererRequest.RELEASE_TEXTURE },
      { key: "b", kind: ERendererRequest.RELEASE_TEXTURE },
      { kind: ERendererRequest.DISPOSE },
      { key: "c", kind: ERendererRequest.RELEASE_TEXTURE },
    ];

    host.take({ kind: ERendererRequest.BATCH, requests });

    expect(released).toEqual(["a:1", "b:1"]);
    expect(disposedAt).toEqual([0]);
  });
});
