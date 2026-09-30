import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { mockDdsFile } from "@xrf/dds/fixtures";
import { Texture, WebGPURenderer } from "three/webgpu";

import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";
import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { ITextureDeviceFixture, mockTextureDevice } from "#/internals/device-fixtures";
import { getWhiteTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { hasTextureData } from "#/texture/texture-data";
import { ITextureTarget } from "#/texture/texture-target";

/** A renderer that uploads nothing, which is all the queue asks of it here. */
const RENDERER: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

/** One request the fetch mock holds, answered when a test says. */
interface IPendingFetch {
  url: string;
  signal: AbortSignal;
  answer: (response: Response) => void;
}

/** @returns A source fetching the file at `url`, with a picture beside it. */
function toSource(url: string): TRendererTextureSource {
  const file: IRendererFetchRequest = { body: "{}", headers: {}, url };

  return { encoding: ERendererTextureEncoding.FETCH, file, picture: { ...file, url: `${url}.png` } };
}

/** Lets an answered fetch be read, which takes the response's body stream a few turns of the event loop. */
async function flush(): Promise<void> {
  for (let turn: number = 0; turn < 5; turn += 1) {
    await new Promise((resolve: (value: unknown) => void) => setTimeout(resolve, 0));
  }
}

function createTextures(): RendererTextures {
  return new RendererTextures(
    () => {},
    () => {}
  );
}

describe("RendererTextures fetching", () => {
  const original: typeof fetch = globalThis.fetch;
  const pending: Array<IPendingFetch> = [];

  beforeEach(() => {
    pending.length = 0;
    globalThis.fetch = (input: string | URL | Request, init?: RequestInit): Promise<Response> =>
      new Promise((answer: (response: Response) => void, reject: (error: unknown) => void): void => {
        const signal: AbortSignal = init?.signal as AbortSignal;

        signal.addEventListener("abort", () => reject(new DOMException("aborted", "AbortError")));
        pending.push({ answer, signal, url: String(input) });
      });
  });

  afterEach(() => {
    globalThis.fetch = original;
  });

  it("holds a key as not uploaded while its file fetches, then uploads it and tells what it came to", async () => {
    const fetched: Array<[string, IRendererTextureFetch]> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (key: string, fetch: IRendererTextureFetch) => fetched.push([key, fetch])
    );
    const target: ITextureTarget = { value: getWhiteTexture() };

    textures.target("brick", getWhiteTexture(), target);
    textures.put("brick", toSource("http://127.0.0.1:1/brick"));

    // A settle waits on it, and whatever samples it waits too: nothing is drawn with it half there.
    expect(textures.isUploaded("brick")).toBe(false);
    expect(textures.hasQueued).toBe(true);
    expect(pending.map((it: IPendingFetch) => it.url)).toEqual(["http://127.0.0.1:1/brick"]);

    pending[0].answer(new Response(mockDdsFile({ height: 4, width: 4 })));
    await flush();
    textures.upload(RENDERER, Infinity);

    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.hasQueued).toBe(false);
    expect(target.value).not.toBe(getWhiteTexture());
    expect(fetched).toHaveLength(1);
    expect(fetched[0][0]).toBe("brick");
    expect(fetched[0][1]).toMatchObject({ failure: null, isDecoded: false, size: { height: 4, width: 4 } });
  });

  it("aborts a fetch in flight when its key is released, and tells nothing of it", async () => {
    const fetched: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (key: string) => fetched.push(key)
    );

    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    textures.release("brick");
    await flush();

    expect(pending[0].signal.aborted).toBe(true);
    expect(fetched).toEqual([]);
    expect(textures.hasQueued).toBe(false);
    expect(textures.isUploaded("brick")).toBe(true);
  });

  it("ignores a fetch answering after its key was put again, and draws only the later put", async () => {
    const fetched: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (key: string, fetch: IRendererTextureFetch) => fetched.push(`${key}:${fetch.bytes}`)
    );
    const target: ITextureTarget = { value: getWhiteTexture() };
    const later: ArrayBuffer = mockDdsFile({ height: 8, width: 8 });

    textures.target("brick", getWhiteTexture(), target);
    textures.put("brick", toSource("http://127.0.0.1:1/first"));
    textures.put("brick", toSource("http://127.0.0.1:1/second"));

    expect(pending[0].signal.aborted).toBe(true);

    // Answered anyway, as a server that never saw the abort would: the answer is for a put since replaced.
    pending[0].answer(new Response(mockDdsFile()));
    pending[1].answer(new Response(later));
    await flush();
    textures.upload(RENDERER, Infinity);

    expect(fetched).toEqual([`brick:${later.byteLength}`]);
    expect((textures.getUploaded("brick")?.image as { width: number }).width).toBe(8);
  });

  it("stops a fetch when bytes are put over it, so the key is not left waiting on it", async () => {
    const fetched: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (key: string) => fetched.push(key)
    );

    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    textures.put("brick", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
    textures.upload(RENDERER, Infinity);
    await flush();

    expect(pending[0].signal.aborted).toBe(true);
    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.hasQueued).toBe(false);
    expect(fetched).toEqual([]);
  });

  it("tells a failed fetch and draws the placeholder, leaving nothing waiting", async () => {
    const fetched: Array<IRendererTextureFetch> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (_: string, fetch: IRendererTextureFetch) => fetched.push(fetch)
    );
    const target: ITextureTarget = { value: getWhiteTexture() };

    textures.target("brick", getWhiteTexture(), target);
    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    pending[0].answer(new Response(JSON.stringify("Failed to read asset 'brick'"), { status: 500 }));
    await flush();

    expect(fetched).toMatchObject([{ failure: "Failed to read asset 'brick'", size: null }]);
    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.hasQueued).toBe(false);
    expect(target.value).toBe(getWhiteTexture());
  });

  it("aborts every fetch in flight as it is disposed", () => {
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {}
    );

    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    textures.put("plaster", toSource("http://127.0.0.1:1/plaster"));
    textures.dispose();

    expect(pending.map((it: IPendingFetch) => it.signal.aborted)).toEqual([true, true]);
    expect(textures.hasQueued).toBe(false);
  });

  // A picture put and then replaced by bytes before it decoded used to leave the key decoding for good: never
  // uploaded, so nothing sampling it ever applied.
  it("is not left loading by a picture superseded before it decoded", async () => {
    Object.assign(globalThis, {
      createImageBitmap: () => new Promise(() => {}),
    });

    try {
      const textures: RendererTextures = new RendererTextures(
        () => {},
        () => {}
      );

      textures.put("brick", { bytes: new ArrayBuffer(4), encoding: ERendererTextureEncoding.IMAGE, type: "image/png" });
      textures.put("brick", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
      textures.upload(RENDERER, Infinity);
      await flush();

      expect(textures.isUploaded("brick")).toBe(true);
      expect(textures.hasQueued).toBe(false);
    } finally {
      Reflect.deleteProperty(globalThis, "createImageBitmap");
    }
  });

  it("lets a fetched texture's bytes go once it is up, and fetches it again to go up after its eviction", async () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const fetched: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (key: string) => fetched.push(key)
    );
    const target: ITextureTarget = { value: getWhiteTexture() };

    textures.target("brick", getWhiteTexture(), target);
    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    pending[0].answer(new Response(mockDdsFile({ height: 8, width: 8 })));
    await flush();
    textures.upload(device.renderer, Infinity);

    const first: Texture = target.value;

    expect(device.uploads.map(([texture, bytes]: [Texture, number]) => [texture, bytes])).toEqual([[first, 32]]);
    expect(hasTextureData(first)).toBe(false);
    expect(textures.listHeldData()).toEqual([]);

    // Its layer holds it now; held again with no layer to fill it from, it has nothing to go up from but its file.
    expect(textures.evict("brick")).toBe(first);
    expect(textures.isUploaded("brick")).toBe(false);

    textures.hold(["brick"]);
    textures.upload(device.renderer, Infinity);

    expect(pending.map((it: IPendingFetch) => it.url)).toEqual([
      "http://127.0.0.1:1/brick",
      "http://127.0.0.1:1/brick",
    ]);
    expect(device.uploads).toHaveLength(1);
    expect(textures.hasQueued).toBe(true);
    expect(target.value).toBe(getWhiteTexture());

    pending[1].answer(new Response(mockDdsFile({ height: 8, width: 8 })));
    await flush();
    textures.upload(device.renderer, Infinity);

    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.isEvicted("brick")).toBe(false);
    expect(target.value).not.toBe(first);
    expect(device.uploads.map(([, bytes]: [Texture, number]) => bytes)).toEqual([32, 32]);
    expect(fetched).toEqual(["brick", "brick"]);
  });

  it("fills an evicted key again from its layer rather than its file, only while something holds it", async () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const rebound: Array<string> = [];
    const restored: Array<[string, Texture]> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      (key: string) => rebound.push(key),
      () => {},
      (_: WebGPURenderer, key: string, texture: Texture): boolean => {
        restored.push([key, texture]);

        return true;
      }
    );
    const target: ITextureTarget = { value: getWhiteTexture() };

    textures.target("brick", getWhiteTexture(), target);
    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    pending[0].answer(new Response(mockDdsFile()));
    await flush();
    textures.upload(device.renderer, Infinity);

    const first: Texture = target.value;

    textures.evict("brick");
    textures.hold(["brick"]);
    textures.letGo(["brick"]);
    textures.upload(device.renderer, Infinity);

    expect(restored).toEqual([]);

    textures.hold(["brick"]);
    textures.upload(device.renderer, Infinity);

    // The texture it was, drawn again at once: nothing fetched, nothing left to go up.
    expect(restored).toEqual([["brick", first]]);
    expect(pending).toHaveLength(1);
    expect(target.value).toBe(first);
    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.hasQueued).toBe(false);
    expect(rebound).toEqual(["brick", "brick"]);
  });

  it("fetches an evicted key again once for all its holds, and nothing more while they stay", async () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const textures: RendererTextures = createTextures();

    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    pending[0].answer(new Response(mockDdsFile()));
    await flush();
    textures.upload(device.renderer, Infinity);
    textures.evict("brick");
    // Two holders at once, and asked about besides: one fetch.
    textures.hold(["brick", "brick"]);
    textures.isUploaded("brick");
    textures.upload(device.renderer, Infinity);
    textures.upload(device.renderer, Infinity);
    pending[1].answer(new Response(mockDdsFile()));
    await flush();

    for (let frame: number = 0; frame < 3; frame += 1) {
      textures.upload(device.renderer, Infinity);
      // Held, so an array's copy of it lets nothing go.
      expect(textures.evict("brick")).toBeNull();
    }

    expect(pending).toHaveLength(2);
    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.hasQueued).toBe(false);
  });

  it("fetches nothing for an evicted key whose hold went before its turn came", async () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const textures: RendererTextures = createTextures();

    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    pending[0].answer(new Response(mockDdsFile()));
    await flush();
    textures.upload(device.renderer, Infinity);
    textures.evict("brick");
    textures.hold(["brick"]);
    textures.letGo(["brick"]);
    textures.upload(device.renderer, Infinity);

    expect(pending).toHaveLength(1);
    expect(textures.isEvicted("brick")).toBe(true);
    expect(textures.hasQueued).toBe(false);
  });

  it("keeps the bytes of a texture handed over, which it could not fetch again, and sends them after its eviction", () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {}
    );

    textures.put("brick", { bytes: mockDdsFile({ height: 8, width: 8 }), encoding: ERendererTextureEncoding.DDS });
    textures.upload(device.renderer, Infinity);

    const texture: Texture = textures.getUploaded("brick") as Texture;

    expect(textures.listHeldData().map((data: ArrayBufferView) => data.byteLength)).toEqual([32]);

    textures.evict("brick");
    textures.hold(["brick"]);
    textures.upload(device.renderer, Infinity);

    expect(pending).toEqual([]);
    expect(device.uploads).toEqual([
      [texture, 32],
      [texture, 32],
    ]);
    expect(textures.getUploaded("brick")).toBe(texture);
  });

  it("holds nothing for a key whose file could not be fetched again, and fetches it no more", async () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const fetched: Array<IRendererTextureFetch> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {},
      (_: string, fetch: IRendererTextureFetch) => fetched.push(fetch)
    );

    textures.put("brick", toSource("http://127.0.0.1:1/brick"));
    pending[0].answer(new Response(mockDdsFile()));
    await flush();
    textures.upload(device.renderer, Infinity);
    textures.evict("brick");
    textures.hold(["brick"]);
    textures.upload(device.renderer, Infinity);
    pending[1].answer(new Response(JSON.stringify("Failed to read asset 'brick'"), { status: 500 }));
    await flush();
    textures.upload(device.renderer, Infinity);

    expect(fetched.map((fetch: IRendererTextureFetch) => fetch.failure)).toEqual([
      null,
      "Failed to read asset 'brick'",
    ]);
    expect(textures.isUploaded("brick")).toBe(true);
    expect(textures.getUploaded("brick")).toBeNull();
    expect(textures.hasQueued).toBe(false);
    expect(pending).toHaveLength(2);
  });
});
