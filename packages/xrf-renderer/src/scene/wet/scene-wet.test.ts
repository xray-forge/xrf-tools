import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { Data3DTexture } from "three/webgpu";

import { ERendererEngine } from "#/contract/renderer-engine";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { SceneWet } from "#/scene/wet/scene-wet";
import { RendererTextures } from "#/texture/renderer-textures";
import { WetUniforms } from "#/uniforms/wet-uniforms";

/** A four by four by two DXT5 volume of one block. */
function createVolume(): ArrayBuffer {
  const bytes: Uint8Array = new Uint8Array(128 + 2 * 16);
  const view: DataView = new DataView(bytes.buffer);

  view.setUint32(0, 0x20534444, true);
  view.setUint32(12, 4, true);
  view.setUint32(16, 4, true);
  view.setUint32(24, 2, true);
  bytes.set([68, 88, 84, 53], 84);
  view.setUint32(112, 0x200000, true);

  return bytes.buffer as ArrayBuffer;
}

const REQUEST = { body: "", headers: {}, url: "volume" };

/** A weather that rains, its surfaces wetted with a volume and a flow. */
function toWeather(): IRendererWeather {
  return {
    effects: {},
    engine: ERendererEngine.VANILLA,
    keyframes: [],
    modifiers: [],
    rain: null,
    sunTable: null,
    textures: {
      "water\\water_SBumpVolume": { encoding: ERendererTextureEncoding.FETCH, file: REQUEST, picture: REQUEST },
    },
    thunder: null,
    wet: { flow: "water\\water_flowing_nmap", splash: "water\\water_SBumpVolume" },
  };
}

function createWet(): { scene: SceneWet; wet: WetUniforms; fetched: jest.Mock } {
  const fetched = jest.fn(async () => new Response(createVolume()));

  globalThis.fetch = fetched as unknown as typeof fetch;

  const wet: WetUniforms = new WetUniforms();

  return {
    fetched,
    scene: new SceneWet(
      new RendererTextures(
        () => {},
        () => {}
      ),
      wet
    ),
    wet,
  };
}

const original: typeof fetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = original;
});

describe("SceneWet", () => {
  it("decodes the splashes' volume into the sampler, once for a weather sent again", async () => {
    const { scene, wet, fetched } = createWet();

    scene.take(toWeather());
    await new Promise((resolve) => setTimeout(resolve, 0));

    const volume = wet.splash.value as Data3DTexture;

    expect(volume.image).toMatchObject({ depth: 2, height: 4, width: 4 });

    scene.take(toWeather());

    expect(fetched).toHaveBeenCalledTimes(1);
    expect(wet.splash.value).toBe(volume);
  });

  it("lets the volume go without a weather", async () => {
    const { scene, wet } = createWet();
    const neutral = wet.splash.value;

    scene.take(toWeather());
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(wet.splash.value).not.toBe(neutral);

    scene.take(null);

    expect(wet.splash.value).toBe(neutral);
  });
});
