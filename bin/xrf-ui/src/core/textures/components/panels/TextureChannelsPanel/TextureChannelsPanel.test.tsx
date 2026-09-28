import { afterAll, beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { ERendererBumpPlane } from "@xrf/renderer";
import { createRendererWorkerStub } from "@xrf/renderer/fixtures";
import { Nullable } from "@xrf/types";

import { ITextureBumpTexels, ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockRendererThread } from "@/fixtures/utils/renderer";
import { AsyncState } from "@/lib/async-state";

import { TEXTURE_CHANNEL_TILES } from "./TextureChannelsPanel.utils";

/** One capture the panel asked for, answered when the test says. */
interface IHeldCapture {
  plane: ERendererBumpPlane;
  resolve: (image: Nullable<ImageBitmap>) => void;
}

/** The side every tile is laid out at, since jsdom lays nothing out. */
const TILE_SIZE: number = 10;

let TextureChannelsPanel: typeof import("./TextureChannelsPanel").TextureChannelsPanel;
let TextureRenderService: typeof import("@/core/textures/services/render").TextureRenderService;
let captures: Array<IHeldCapture> = [];

/** A picture as the renderer answers one: its size is what the tile takes. */
function mockImage(width: number): ImageBitmap {
  return { close: jest.fn(), height: TILE_SIZE, width } as unknown as ImageBitmap;
}

function mockFiles(): ITextureSurfaceFiles {
  const half = { bytes: new ArrayBuffer(0), height: 64, isDecoded: false, width: 64 };

  return { aspect: 1, base: null, bump: { bump: half, companion: half } };
}

function renderPanel(): { container: Container; view: RenderResult } {
  const container: Container = mockContainer([
    TextureSelectionService,
    TextureSurfaceService,
    TextureViewService,
    TextureRenderService,
  ]);

  runInAction(() => {
    container.get(TextureSelectionService).selected = AsyncState.ready(
      mockTextureDescription(undefined, { material: mockMaterialDescriptor() })
    );
    container.get(TextureSurfaceService).files = AsyncState.ready(mockFiles());
  });

  return { container, view: renderWithProviders(<TextureChannelsPanel />, { container }) };
}

function getTile(view: RenderResult, plane: ERendererBumpPlane): HTMLCanvasElement {
  return view.getByTestId(`texture-channel-${plane}`) as HTMLCanvasElement;
}

async function answer(plane: ERendererBumpPlane, image: ImageBitmap, which: number = -1): Promise<void> {
  const capture: IHeldCapture = captures.filter((it: IHeldCapture) => it.plane === plane).at(which) as IHeldCapture;

  await act(async () => capture.resolve(image));
}

beforeAll(async () => {
  mockRendererThread(() => createRendererWorkerStub().worker);

  ({ TextureChannelsPanel } = await import("./TextureChannelsPanel"));
  ({ TextureRenderService } = await import("@/core/textures/services/render"));

  jest
    .spyOn(TextureRenderService.prototype, "captureBumpPlane")
    .mockImplementation(
      (plane: ERendererBumpPlane) =>
        new Promise((resolve: (image: Nullable<ImageBitmap>) => void) => captures.push({ plane, resolve }))
    );

  for (const side of ["clientWidth", "clientHeight"]) {
    Object.defineProperty(HTMLCanvasElement.prototype, side, { configurable: true, get: () => TILE_SIZE });
  }
});

afterAll(() => {
  for (const side of ["clientWidth", "clientHeight"]) {
    delete (HTMLCanvasElement.prototype as unknown as Record<string, unknown>)[side];
  }
});

beforeEach(() => {
  captures = [];
});

describe("TextureChannelsPanel", () => {
  it("asks for every plane at its tile's size, and draws each one it is answered", async () => {
    const { view } = renderPanel();

    expect(captures.map((it) => it.plane)).toEqual([
      ERendererBumpPlane.BUMP,
      ERendererBumpPlane.COMPANION,
      ERendererBumpPlane.NORMAL,
      ERendererBumpPlane.GLOSS,
      ERendererBumpPlane.HEIGHT,
    ]);

    await answer(ERendererBumpPlane.NORMAL, mockImage(7));

    expect(getTile(view, ERendererBumpPlane.NORMAL).width).toBe(7);
    expect(view.getByRole("img", { name: "Normal" })).toBeInTheDocument();
  });

  // A ref callback made anew each render let go of every tile and its capture on each re-render, so hovering a tile
  // left every plane blank.
  it("draws an answer that arrives after the panel drew again", async () => {
    const { container, view } = renderPanel();

    act(() =>
      runInAction(() => {
        container.get(TextureSurfaceService).bumpTexels = {} as ITextureBumpTexels;
      })
    );

    await answer(ERendererBumpPlane.GLOSS, mockImage(6));

    expect(getTile(view, ERendererBumpPlane.GLOSS).width).toBe(6);
  });

  it("never paints an older answer over a newer one", async () => {
    const { container, view } = renderPanel();
    const stale: ImageBitmap = mockImage(7);

    act(() =>
      runInAction(() => {
        container.get(TextureSurfaceService).files = AsyncState.ready(mockFiles());
      })
    );

    await answer(ERendererBumpPlane.BUMP, mockImage(9));
    await answer(ERendererBumpPlane.BUMP, stale, 0);

    expect(getTile(view, ERendererBumpPlane.BUMP).width).toBe(9);
    expect(stale.close).toHaveBeenCalled();
  });

  it("says why there are no planes once the renderer has stopped", () => {
    const { container, view } = renderPanel();

    act(() =>
      runInAction(() => {
        container.get(TextureRenderService).failure = "Device lost";
      })
    );

    expect(view.getByTestId("texture-channels-failure")).toHaveTextContent("Device lost");
  });

  // The readout is read from the cpu texels, so a machine without a gpu the renderer can use still reads every texel.
  it("keeps the readout and every tile to hover once the renderer has stopped", () => {
    const { container, view } = renderPanel();

    act(() =>
      runInAction(() => {
        container.get(TextureSurfaceService).bumpTexels = {} as ITextureBumpTexels;
        container.get(TextureRenderService).failure = "Device lost";
      })
    );

    expect(view.getByTestId("texture-channels-readout")).toHaveTextContent("At");

    for (const plane of Object.values(ERendererBumpPlane)) {
      expect(getTile(view, plane)).toBeInTheDocument();
    }
  });

  it("draws every plane again once a renderer starts after the one that stopped", () => {
    const { container } = renderPanel();

    act(() =>
      runInAction(() => {
        container.get(TextureRenderService).failure = "Device lost";
      })
    );

    captures = [];

    act(() =>
      runInAction(() => {
        container.get(TextureRenderService).failure = null;
      })
    );

    expect(captures).toHaveLength(TEXTURE_CHANNEL_TILES.length);
  });
});
