import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { RenderTarget, Vector2, WebGPURenderer } from "three/webgpu";

import { RendererCaptures } from "#/capture/renderer-captures";
import { ERendererBumpPlane } from "#/contract/renderer-bump-plane";
import { ERendererCaptureSource, TRendererCaptureSource } from "#/contract/renderer-capture-source";
import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { PresentPass } from "#/pass/present-pass";
import { RendererTextures } from "#/texture/renderer-textures";

const FRAME: TRendererCaptureSource = { kind: ERendererCaptureSource.FRAME, view: ERendererDebugView.FINAL };

const BUMP: TRendererCaptureSource = {
  bump: "bump",
  companion: "companion",
  height: 4,
  kind: ERendererCaptureSource.BUMP_PLANE,
  plane: ERendererBumpPlane.BUMP,
  width: 4,
};

interface IFakeBitmap {
  width: number;
  close: jest.Mock<() => void>;
}

interface IFakeCaptures {
  captures: RendererCaptures;
  renderer: WebGPURenderer;
  replies: Array<[number, Nullable<IFakeBitmap>]>;
  drawn: Array<RenderTarget>;
  /** How many holds each texture key has. */
  holds: Map<string, number>;
  /** Fails every read back from now on. */
  failReads(): void;
}

function createCaptures(uploaded: ReadonlySet<string> = new Set()): IFakeCaptures {
  const replies: Array<[number, Nullable<IFakeBitmap>]> = [];
  const drawn: Array<RenderTarget> = [];
  let isFailing: boolean = false;
  const present = {
    draw: (_renderer: WebGPURenderer, _view: ERendererDebugView, target: RenderTarget): void => {
      drawn.push(target);
    },
  };
  const holds: Map<string, number> = new Map();
  const textures = {
    hold: (keys: Iterable<string>): void =>
      [...keys].forEach((key: string) => holds.set(key, (holds.get(key) ?? 0) + 1)),
    isUploaded: (key: string): boolean => uploaded.has(key),
    letGo: (keys: Iterable<string>): void =>
      [...keys].forEach((key: string) => holds.set(key, (holds.get(key) ?? 0) - 1)),
  };
  const renderer = {
    readRenderTargetPixelsAsync: (_target: RenderTarget, _x: number, _y: number, width: number, height: number) =>
      isFailing ? Promise.reject(new Error("lost")) : Promise.resolve(new Uint8Array(width * height * 4)),
  };

  return {
    captures: new RendererCaptures(
      present as unknown as PresentPass,
      textures as unknown as RendererTextures,
      (id: number, image: Nullable<ImageBitmap>) => replies.push([id, image as Nullable<IFakeBitmap>])
    ),
    drawn,
    failReads: () => {
      isFailing = true;
    },
    holds,
    renderer: renderer as unknown as WebGPURenderer,
    replies,
  };
}

async function settle(): Promise<void> {
  for (let turn: number = 0; turn < 8; turn += 1) {
    await Promise.resolve();
  }
}

describe("RendererCaptures", () => {
  beforeEach(() => {
    Object.assign(globalThis, {
      createImageBitmap: async (data: { width: number }): Promise<IFakeBitmap> => ({
        close: jest.fn(),
        width: data.width,
      }),
      ImageData: class {
        public readonly width: number;

        public constructor(_: Uint8ClampedArray, width: number) {
          this.width = width;
        }
      },
    });
  });

  afterEach(() => {
    Reflect.deleteProperty(globalThis, "createImageBitmap");
    Reflect.deleteProperty(globalThis, "ImageData");
  });

  it("answers a frame capture with nothing while no view is attached", () => {
    const { captures, renderer, replies }: IFakeCaptures = createCaptures();

    captures.push(1, FRAME);
    captures.answer(renderer, false, null);

    expect(replies).toEqual([[1, null]]);
    expect(captures.hasPending).toBe(false);
  });

  // A frame still settling, or one whose targets were just allocated, reads back half changed or cleared.
  it("keeps a frame capture waiting for a frame that can be read", () => {
    const { captures, renderer, replies, drawn }: IFakeCaptures = createCaptures();

    captures.push(1, FRAME);
    captures.answer(renderer, true, null);

    expect(replies).toEqual([]);
    expect(drawn).toEqual([]);
    expect(captures.hasPending).toBe(true);
  });

  it("draws a frame capture at the drawing's size into a target of its own and hands its picture back", async () => {
    const { captures, renderer, replies, drawn }: IFakeCaptures = createCaptures();

    captures.push(3, FRAME);
    captures.answer(renderer, true, new Vector2(8, 2));
    await settle();

    expect(drawn.map((target: RenderTarget) => [target.width, target.height])).toEqual([[8, 2]]);
    expect(replies).toEqual([[3, expect.objectContaining({ width: 8 })]]);
    expect(captures.hasPending).toBe(false);
  });

  it("answers with nothing where the read back fails", async () => {
    const { captures, renderer, replies, failReads }: IFakeCaptures = createCaptures();

    failReads();
    captures.push(1, FRAME);
    captures.answer(renderer, true, new Vector2(4, 4));
    await settle();

    expect(replies).toEqual([[1, null]]);
  });

  it("keeps a bump plane waiting until both halves of its pair are on the GPU, with or without a view", () => {
    const { captures, renderer, replies }: IFakeCaptures = createCaptures(new Set(["bump"]));

    captures.push(1, BUMP);
    captures.answer(renderer, false, null);
    captures.answer(renderer, true, new Vector2(4, 4));

    expect(replies).toEqual([]);
    expect(captures.hasPending).toBe(true);
  });

  // Asking whether a half is up brings nothing back: a half an array's layer holds comes back only for a hold.
  it("holds a waiting bump plane's pair until it is drawn or the renderer goes", () => {
    const waiting: IFakeCaptures = createCaptures(new Set(["bump"]));

    waiting.captures.push(1, BUMP);
    waiting.captures.answer(waiting.renderer, false, null);

    expect(Object.fromEntries(waiting.holds)).toEqual({ bump: 1, companion: 1 });

    waiting.captures.dispose();

    expect(Object.fromEntries(waiting.holds)).toEqual({ bump: 0, companion: 0 });
  });

  // The renderer going away is the answer: its client refuses every capture waiting.
  it("answers nothing once disposed, and closes a picture read back after", async () => {
    const { captures, renderer, replies }: IFakeCaptures = createCaptures();
    const closed: Array<IFakeBitmap> = [];
    const decode: unknown = Reflect.get(globalThis, "createImageBitmap");

    Object.assign(globalThis, {
      createImageBitmap: async (data: { width: number }): Promise<IFakeBitmap> => {
        const bitmap: IFakeBitmap = await (decode as (data: { width: number }) => Promise<IFakeBitmap>)(data);

        closed.push(bitmap);

        return bitmap;
      },
    });
    captures.push(1, FRAME);
    captures.push(2, FRAME);
    captures.answer(renderer, true, new Vector2(4, 4));
    captures.dispose();
    await settle();

    expect(replies).toEqual([]);
    expect(captures.hasPending).toBe(false);
    expect(closed.map((bitmap: IFakeBitmap) => bitmap.close.mock.calls.length)).toEqual([1, 1]);
  });
});
