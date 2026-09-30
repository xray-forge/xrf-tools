import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { mockDdsFile, mockUndecodableDdsFile } from "@xrf/dds/fixtures";

import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { fetchRendererTexture } from "#/texture/fetch-renderer-texture";
import { IRendererTextureLoad } from "#/texture/renderer-texture-load";

const FILE: IRendererFetchRequest = {
  body: '{"logicalPath":"a.dds"}',
  headers: { Authorization: "Bearer token" },
  url: "http://127.0.0.1:1/assets/read_asset",
};

const PICTURE: IRendererFetchRequest = { ...FILE, url: "http://127.0.0.1:1/textures/read_texture" };

function mockFetch(answers: Record<string, () => Response>): jest.Mock<typeof fetch> {
  const mock: jest.Mock<typeof fetch> = jest.fn(async (input: string | URL | Request): Promise<Response> => {
    const answer: (() => Response) | undefined = answers[String(input)];

    if (!answer) {
      throw new TypeError("Failed to fetch");
    }

    return answer();
  });

  globalThis.fetch = mock;

  return mock;
}

describe("fetchRendererTexture", () => {
  const original: typeof fetch = globalThis.fetch;

  afterEach(() => {
    globalThis.fetch = original;
    Reflect.deleteProperty(globalThis, "createImageBitmap");
  });

  it("posts the file's request as given and reads the answer as stored, with its size", async () => {
    const file: ArrayBuffer = mockDdsFile({ height: 8, mipmapCount: 4, width: 8 });
    const fetched: jest.Mock<typeof fetch> = mockFetch({ [FILE.url]: () => new Response(file) });

    const load: IRendererTextureLoad = await fetchRendererTexture(FILE, PICTURE, new AbortController().signal);

    expect(fetched).toHaveBeenCalledTimes(1);
    expect(fetched.mock.calls[0][1]).toMatchObject({ body: FILE.body, headers: FILE.headers, method: "POST" });
    expect(load.texture).not.toBeNull();
    expect(load.fetch).toMatchObject({
      bytes: file.byteLength,
      failure: null,
      isDecoded: false,
      size: { height: 8, levels: 4, width: 8 },
    });
  });

  it("fetches the picture where the reader refuses the file, and decodes it as its answer names", async () => {
    const picture: ArrayBuffer = new Uint8Array([1, 2, 3]).buffer;
    const decoded: Array<Blob> = [];

    Object.assign(globalThis, {
      createImageBitmap: async (blob: Blob) => {
        decoded.push(blob);

        return { close: () => {}, height: 2, width: 4 };
      },
    });
    mockFetch({
      [FILE.url]: () => new Response(mockUndecodableDdsFile()),
      [PICTURE.url]: () => new Response(picture, { headers: { "content-type": "image/png" } }),
    });

    const load: IRendererTextureLoad = await fetchRendererTexture(FILE, PICTURE, new AbortController().signal);

    expect(load.texture).not.toBeNull();
    expect(decoded.map((blob: Blob) => blob.type)).toEqual(["image/png"]);
    expect(load.fetch).toMatchObject({
      bytes: mockUndecodableDdsFile().byteLength + picture.byteLength,
      failure: null,
      isDecoded: true,
      size: { height: 2, levels: 1, width: 4 },
    });
  });

  it("answers the server's own message for a refused request, rather than rejecting", async () => {
    mockFetch({
      [FILE.url]: () => new Response(JSON.stringify("Failed to read asset 'a.dds': not found"), { status: 500 }),
    });

    const load: IRendererTextureLoad = await fetchRendererTexture(FILE, PICTURE, new AbortController().signal);

    expect(load.texture).toBeNull();
    expect(load.fetch).toMatchObject({ failure: "Failed to read asset 'a.dds': not found", size: null });
  });

  it("answers the status where a refusal carries no message", async () => {
    mockFetch({ [FILE.url]: () => new Response("<html>", { status: 401 }) });

    const load: IRendererTextureLoad = await fetchRendererTexture(FILE, PICTURE, new AbortController().signal);

    expect(load.fetch.failure).toBe("The texture's server answered 401");
  });

  it("answers a transport that never answered as a failure", async () => {
    mockFetch({});

    const load: IRendererTextureLoad = await fetchRendererTexture(FILE, PICTURE, new AbortController().signal);

    expect(load.texture).toBeNull();
    expect(load.fetch.failure).toBe("Failed to fetch");
  });

  it("passes its signal to the fetch, so an abort ends the request", async () => {
    const controller: AbortController = new AbortController();
    const fetched: jest.Mock<typeof fetch> = mockFetch({ [FILE.url]: () => new Response(mockDdsFile()) });

    await fetchRendererTexture(FILE, PICTURE, controller.signal);

    expect(fetched.mock.calls[0][1]?.signal).toBe(controller.signal);
  });
});
