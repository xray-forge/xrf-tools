import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { ERendererEngine } from "#/contract/renderer-engine";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRequest, TRendererRequest } from "#/contract/renderer-request";
import { ERendererResponse, TRendererResponse } from "#/contract/renderer-response";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { ERendererWeatherTransition } from "#/contract/weather/renderer-weather-transition";
import { RendererDevice } from "#/device/renderer-device";
import { RendererDeviceFailure } from "#/device/renderer-device-failure";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";
import { TRendererFrameScheduler } from "#/host/renderer-frame-scheduler";
import { RendererHost } from "#/host/renderer-host";
import { RendererView } from "#/host/renderer-view";
import { RendererScene } from "#/scene/renderer-scene";

const SETTINGS: IRendererSettings = {
  backdrop: null,
  debugView: ERendererDebugView.FINAL,
  features: RENDERER_PRESETS[ERendererPreset.BASE],
  hemiStrength: 1,
  isBumped: true,
  isGpuTimed: false,
  isLit: true,
  isSkyDrawn: false,
  isSkyHazed: false,
  isTextured: true,
  isWireframe: false,
  pacing: DEFAULT_RENDER_FRAME_PACING,
  tonemapScale: 1,
};

function createHost(schedule: TRendererFrameScheduler = () => 0): [RendererHost, Array<TRendererResponse>] {
  const replies: Array<TRendererResponse> = [];
  const host: RendererHost = new RendererHost(
    (response: TRendererResponse) => {
      if (response.kind !== ERendererResponse.CURSOR) {
        replies.push(response);
      }
    },
    schedule,
    () => {}
  );

  return [host, replies];
}

function createDevice(): RendererDevice {
  return {
    describe: () => ({}),
    dispose: () => {},
    headless: {},
    renderer: { backend: { device: { limits: {} } }, hasFeature: () => true, setCanvasTarget: () => {} },
  } as unknown as RendererDevice;
}

interface IDrawingHost {
  draw(): boolean;
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

  // A capture taken on the frame the water was switched on for would show none: it joins once compiled.
  it("answers a settle only with a frame drawn with every stage asked for, one joining among them", async () => {
    const frames: Array<(now: number) => void> = [];
    const [host, replies]: [RendererHost, Array<TRendererResponse>] = createHost((callback: (now: number) => void) =>
      frames.push(callback)
    );
    const { features } = SETTINGS;
    const unwatered: IRendererSettings = {
      ...SETTINGS,
      features: { ...features, water: { ...features.water, isEnabled: false } },
    };
    const watered: IRendererSettings = {
      ...SETTINGS,
      features: { ...features, water: { ...features.water, isEnabled: true } },
    };
    let now: number = 0;

    function runFrame(): void {
      now += 1000;
      frames.shift()?.(now);
    }

    function toSettled(): Array<number> {
      return replies.flatMap((reply: TRendererResponse) =>
        reply.kind === ERendererResponse.SETTLED ? [reply.id] : []
      );
    }

    jest.spyOn(RendererDevice, "open").mockResolvedValue(createDevice());
    jest.spyOn(RendererView.prototype, "show").mockImplementation(() => {});
    jest.spyOn(RendererView.prototype, "hide").mockImplementation(() => {});
    jest.spyOn(RendererHost.prototype as unknown as IDrawingHost, "draw").mockReturnValue(false);

    host.take({ kind: ERendererRequest.START, settings: unwatered });
    await settle();
    host.take({
      canvas: {} as OffscreenCanvas,
      height: 1,
      kind: ERendererRequest.ATTACH_VIEW,
      pixelRatio: 1,
      width: 1,
    });
    host.take({ id: 1, kind: ERendererRequest.SETTLE });
    runFrame();

    expect(toSettled()).toEqual([1]);

    host.take({ kind: ERendererRequest.CONFIGURE, settings: watered });
    host.take({ id: 2, kind: ERendererRequest.SETTLE });
    runFrame();

    expect(toSettled()).toEqual([1]);

    runFrame();

    expect(toSettled()).toEqual([1, 2]);

    host.take({ kind: ERendererRequest.DISPOSE });
  });

  // A keyframe set by hand sends its keyframes alone: the rest is the weather the consumer handed over before.
  it("lays what changed of a weather over the one it holds", () => {
    const [host] = createHost();
    const taken: Array<Nullable<IRendererWeather>> = [];
    const weather: IRendererWeather = {
      effects: {},
      engine: ERendererEngine.VANILLA,
      keyframes: [],
      modifiers: [],
      rain: { drop: null, streak: "fx\fx_rain" },
      sunTable: null,
      textures: {},
      thunder: null,
      wet: null,
    };
    const keyframes: IRendererWeather["keyframes"] = [];

    jest.spyOn(RendererScene.prototype, "takeWeather").mockImplementation((it) => void taken.push(it));

    host.take({ kind: ERendererRequest.WEATHER, transition: ERendererWeatherTransition.CUT, weather });
    host.take({ kind: ERendererRequest.WEATHER, transition: ERendererWeatherTransition.EASE, weather: { keyframes } });
    host.take({ kind: ERendererRequest.WEATHER, transition: ERendererWeatherTransition.CUT, weather: null });

    expect(taken[1]).toEqual({ ...weather, keyframes });
    expect(taken[1]?.keyframes).toBe(keyframes);
    expect(taken[1]?.rain).toBe(taken[0]?.rain);
    expect(taken[2]).toBeNull();

    host.take({ kind: ERendererRequest.DISPOSE });
  });
});
