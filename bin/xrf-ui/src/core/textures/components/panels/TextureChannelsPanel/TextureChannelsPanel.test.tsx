import { afterAll, beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ETextureBumpPlane } from "@/core/textures/lib/texture-bump-plane";
import { ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { AsyncState } from "@/lib/async-state";

import { TextureChannelsPanel } from "./TextureChannelsPanel";

/** The side every tile is laid out at, since jsdom lays nothing out. */
const TILE_SIZE: number = 10;

/** What a 2d context was asked to draw, since jsdom draws nothing. */
let draws: Array<{ canvas: HTMLCanvasElement; source: HTMLCanvasElement }> = [];

function mockFiles(width: number = 4): ITextureSurfaceFiles {
  const half = { data: new Uint8Array(width * 2 * 4), height: 2, width };

  return { aspect: 1, bump: { bump: half, companion: half } };
}

function renderPanel(): { container: Container; view: RenderResult } {
  const container: Container = mockContainer([TextureSelectionService, TextureSurfaceService, TextureViewService]);

  runInAction(() => {
    container.get(TextureSelectionService).selected = AsyncState.ready(
      mockTextureDescription(undefined, { material: mockMaterialDescriptor() })
    );
    container.get(TextureSurfaceService).files = AsyncState.ready(mockFiles());
  });

  return { container, view: renderWithProviders(<TextureChannelsPanel />, { container }) };
}

/** A 2d context that keeps what it was asked to draw into `canvas`. */
function mockContext(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  return {
    createImageData: (width: number, height: number) => ({ data: new Uint8ClampedArray(width * height * 4) }),
    drawImage: (source: HTMLCanvasElement) => draws.push({ canvas, source }),
    putImageData: () => undefined,
  } as unknown as CanvasRenderingContext2D;
}

function getTile(view: RenderResult, plane: ETextureBumpPlane): HTMLCanvasElement {
  return view.getByTestId(`texture-channel-${plane}`) as HTMLCanvasElement;
}

beforeAll(() => {
  jest.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(function (this: HTMLCanvasElement) {
    return mockContext(this);
  } as never);

  for (const side of ["clientWidth", "clientHeight"]) {
    Object.defineProperty(HTMLCanvasElement.prototype, side, { configurable: true, get: () => TILE_SIZE });
  }
});

afterAll(() => {
  jest.restoreAllMocks();

  for (const side of ["clientWidth", "clientHeight"]) {
    delete (HTMLCanvasElement.prototype as unknown as Record<string, unknown>)[side];
  }
});

beforeEach(() => {
  draws = [];
});

describe("TextureChannelsPanel", () => {
  it("draws every plane at its tile's size, from the pair at its own", () => {
    const { view } = renderPanel();

    for (const plane of Object.values(ETextureBumpPlane)) {
      const tile: HTMLCanvasElement = getTile(view, plane);
      const drawn = draws.filter((it) => it.canvas === tile);

      expect(tile.width).toBe(TILE_SIZE * window.devicePixelRatio);
      expect(drawn).toHaveLength(1);
      expect([drawn[0].source.width, drawn[0].source.height]).toEqual([4, 2]);
    }

    expect(view.getByRole("img", { name: "Normal" })).toBeInTheDocument();
  });

  it("draws every plane again for the next pair", () => {
    const { container, view } = renderPanel();

    draws = [];

    act(() =>
      runInAction(() => {
        container.get(TextureSurfaceService).files = AsyncState.ready(mockFiles(8));
      })
    );

    const drawn = draws.filter((it) => it.canvas === getTile(view, ETextureBumpPlane.GLOSS));

    expect(drawn.at(-1)?.source.width).toBe(8);
  });

  it("reads a texel under the pointer from the pair", () => {
    const { view } = renderPanel();

    expect(view.getByTestId("texture-channels-readout")).toHaveTextContent("At");
  });
});
