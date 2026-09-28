import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { RendererClient } from "#/client/renderer-client";
import { ERendererCaptureSource, TRendererCaptureSource } from "#/contract/renderer-capture-source";
import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRequest, TRendererRequest } from "#/contract/renderer-request";
import { ERendererResponse, TRendererResponse } from "#/contract/renderer-response";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";

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

const FRAME: TRendererCaptureSource = { kind: ERendererCaptureSource.FRAME, view: ERendererDebugView.FINAL };

interface IFakeWorker {
  worker: Worker;
  posts: Array<TRendererRequest>;
  respond(response: TRendererResponse): void;
  crash(message: string): void;
  isTerminated(): boolean;
}

function createWorker(): IFakeWorker {
  const posts: Array<TRendererRequest> = [];
  let isTerminated: boolean = false;
  const worker = {
    onerror: null as Nullable<(event: ErrorEvent) => void>,
    onmessage: null as Nullable<(event: MessageEvent<TRendererResponse>) => void>,
    postMessage: (request: TRendererRequest): void => {
      posts.push(request);
    },
    terminate: (): void => {
      isTerminated = true;
    },
  };

  return {
    crash: (message: string) => worker.onerror?.({ message } as ErrorEvent),
    isTerminated: () => isTerminated,
    posts,
    respond: (response: TRendererResponse) => worker.onmessage?.({ data: response } as MessageEvent),
    worker: worker as unknown as Worker,
  };
}

function listIds(posts: ReadonlyArray<TRendererRequest>, kind: ERendererRequest.SETTLE | ERendererRequest.CAPTURE) {
  return posts
    .flatMap((post: TRendererRequest) => (post.kind === ERendererRequest.BATCH ? post.requests : [post]))
    .flatMap((request: TRendererRequest) => (request.kind === kind ? [request.id] : []));
}

async function flush(): Promise<void> {
  await Promise.resolve();
}

function toImage(): ImageBitmap {
  return { close: jest.fn() } as unknown as ImageBitmap;
}

describe("RendererClient", () => {
  it("starts the renderer once, as the first thing it says", async () => {
    const fake: IFakeWorker = createWorker();

    new RendererClient({ settings: SETTINGS, worker: fake.worker });
    await flush();

    expect(fake.posts).toEqual([{ kind: ERendererRequest.START, settings: SETTINGS }]);
  });

  it("posts what one run asks for as one batch, and what follows an await as another", async () => {
    const fake: IFakeWorker = createWorker();
    const client: RendererClient = new RendererClient({ settings: SETTINGS, worker: fake.worker });

    client.releaseTexture("a");
    client.releaseGeometry("b");
    await flush();
    client.releaseObject("c");
    await flush();

    expect(fake.posts).toEqual([
      {
        kind: ERendererRequest.BATCH,
        requests: [
          { kind: ERendererRequest.START, settings: SETTINGS },
          { key: "a", kind: ERendererRequest.RELEASE_TEXTURE },
          { key: "b", kind: ERendererRequest.RELEASE_GEOMETRY },
        ],
      },
      { key: "c", kind: ERendererRequest.RELEASE_OBJECT },
    ]);
  });

  it("tells the consumer what each fetched texture came to, by its key", () => {
    const fake: IFakeWorker = createWorker();
    const fetched: Array<[string, IRendererTextureFetch]> = [];
    const fetch: IRendererTextureFetch = { bytes: 12, duration: 3, failure: null, isDecoded: false, size: null };

    new RendererClient({
      onTextureFetched: (key: string, it: IRendererTextureFetch) => fetched.push([key, it]),
      settings: SETTINGS,
      worker: fake.worker,
    });
    fake.respond({ fetch, key: "brick", kind: ERendererResponse.TEXTURE_FETCHED });

    expect(fetched).toEqual([["brick", fetch]]);
  });

  it("answers each capture and settle by the id it asked under", async () => {
    const fake: IFakeWorker = createWorker();
    const client: RendererClient = new RendererClient({ settings: SETTINGS, worker: fake.worker });
    const image: ImageBitmap = toImage();
    const stray: ImageBitmap = toImage();
    const first: Promise<Nullable<ImageBitmap>> = client.capture(FRAME);
    const second: Promise<Nullable<ImageBitmap>> = client.capture(FRAME);
    const settled: jest.Mock<() => void> = jest.fn();

    void client.settle().then(settled);
    await flush();

    const [firstId, secondId] = listIds(fake.posts, ERendererRequest.CAPTURE);

    fake.respond({ id: secondId, image, kind: ERendererResponse.CAPTURED });
    fake.respond({ id: firstId, image: null, kind: ERendererResponse.CAPTURED });
    fake.respond({ id: 99, image: stray, kind: ERendererResponse.CAPTURED });
    fake.respond({ id: listIds(fake.posts, ERendererRequest.SETTLE)[0], kind: ERendererResponse.SETTLED });

    await expect(first).resolves.toBeNull();
    await expect(second).resolves.toBe(image);
    await flush();

    expect(settled).toHaveBeenCalledTimes(1);
    // A picture nobody waits for any more is let go.
    expect(stray.close).toHaveBeenCalledTimes(1);
  });

  it("refuses every settle and capture waiting once the renderer failed, and every later one at once", async () => {
    const fake: IFakeWorker = createWorker();
    const onFailed: jest.Mock<(reason: string) => void> = jest.fn();
    const client: RendererClient = new RendererClient({ onFailed, settings: SETTINGS, worker: fake.worker });
    const settled: Promise<void> = client.settle();
    const captured: Promise<Nullable<ImageBitmap>> = client.capture(FRAME);

    await flush();
    fake.respond({ kind: ERendererResponse.FAILED, reason: "The GPU device was lost: gone" });

    await expect(settled).rejects.toThrow("The GPU device was lost: gone");
    await expect(captured).rejects.toThrow("The GPU device was lost: gone");
    await expect(client.settle()).rejects.toThrow("The GPU device was lost: gone");
    await expect(client.capture(FRAME)).rejects.toThrow("The GPU device was lost: gone");
    expect(onFailed).toHaveBeenCalledWith("The GPU device was lost: gone");
  });

  it("fails on its worker's own error as on the renderer's, and tells the consumer once", async () => {
    const fake: IFakeWorker = createWorker();
    const onFailed: jest.Mock<(reason: string) => void> = jest.fn();
    const client: RendererClient = new RendererClient({ onFailed, settings: SETTINGS, worker: fake.worker });
    const settled: Promise<void> = client.settle();

    fake.crash("out of memory");
    fake.respond({ kind: ERendererResponse.FAILED, reason: "later" });

    await expect(settled).rejects.toThrow("The renderer worker failed: out of memory");
    expect(onFailed.mock.calls).toEqual([["The renderer worker failed: out of memory"]]);
  });

  it("takes nothing once failed: posts nothing, takes no canvas, and passes nothing more on", async () => {
    const fake: IFakeWorker = createWorker();
    const onReport: jest.Mock<() => void> = jest.fn();
    const client: RendererClient = new RendererClient({ onReport, settings: SETTINGS, worker: fake.worker });
    const transfer: jest.Mock<() => OffscreenCanvas> = jest.fn(() => ({}) as OffscreenCanvas);
    const image: ImageBitmap = toImage();

    await flush();
    fake.crash("out of memory");
    fake.posts.length = 0;
    client.releaseTexture("a");
    client.attach({
      canvas: { transferControlToOffscreen: transfer } as unknown as HTMLCanvasElement,
      dispose: () => {},
      height: 1,
      observe: () => () => {},
      pixelRatio: 1,
      width: 1,
    });
    await flush();
    // A worker failing on its own may talk on.
    fake.respond({ kind: ERendererResponse.REPORT, report: {} as never });
    fake.respond({ id: 1, image, kind: ERendererResponse.CAPTURED });

    expect(fake.posts).toEqual([]);
    expect(transfer).not.toHaveBeenCalled();
    expect(onReport).not.toHaveBeenCalled();
    expect(image.close).toHaveBeenCalledTimes(1);
  });

  it("answers every settle and capture asked once disposed at once, and posts nothing more", async () => {
    const fake: IFakeWorker = createWorker();
    const client: RendererClient = new RendererClient({ settings: SETTINGS, worker: fake.worker });

    client.dispose();
    fake.posts.length = 0;

    await expect(client.settle()).resolves.toBeUndefined();
    await expect(client.capture(FRAME)).resolves.toBeNull();
    client.releaseTexture("a");
    client.dispose();
    await flush();

    expect(fake.posts).toEqual([]);
  });

  it("answers what waits once disposed, and lets the thread go", async () => {
    const fake: IFakeWorker = createWorker();
    const client: RendererClient = new RendererClient({ settings: SETTINGS, worker: fake.worker });
    const settled: Promise<void> = client.settle();
    const captured: Promise<Nullable<ImageBitmap>> = client.capture(FRAME);

    client.dispose();

    await expect(settled).resolves.toBeUndefined();
    await expect(captured).resolves.toBeNull();
    expect(fake.isTerminated()).toBe(true);
    expect(fake.posts.at(-1)).toMatchObject({
      requests: expect.arrayContaining([{ kind: ERendererRequest.DISPOSE }]),
    });
  });
});
