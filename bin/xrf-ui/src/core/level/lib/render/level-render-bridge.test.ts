import { describe, expect, it, jest } from "@jest/globals";

import { LevelRenderBridge } from "@/core/level/lib/render/level-render-bridge";
import {
  ILevelSectorChange,
  ILevelSectorSource,
  ILevelTextureSupply,
  ILevelTextureSupplyChange,
  TLevelSectorListener,
  TLevelTextureSupplyListener,
} from "@/core/level/lib/render/level-render-protocol";
import { ILevelRenderer } from "@/core/level/lib/render/level-renderer";

function mockRenderer(): ILevelRenderer & {
  delivered: Array<ILevelSectorChange>;
  supplied: Array<ILevelTextureSupplyChange>;
} {
  const delivered: Array<ILevelSectorChange> = [];
  const supplied: Array<ILevelTextureSupplyChange> = [];

  return {
    deliver: (change: ILevelSectorChange): void => void delivered.push(change),
    delivered,
    dispose: jest.fn(),
    measure: () => Promise.resolve(new Map()),
    open: jest.fn(),
    setView: jest.fn(),
    supplied,
    supply: (change: ILevelTextureSupplyChange): void => void supplied.push(change),
  };
}

function mockSource<TListener>(): {
  source: { subscribe(listener: TListener): () => void };
  listeners: Set<TListener>;
} {
  const listeners: Set<TListener> = new Set();

  return {
    listeners,
    source: {
      subscribe: (listener: TListener): (() => void) => {
        listeners.add(listener);

        return (): void => {
          listeners.delete(listener);
        };
      },
    },
  };
}

describe("LevelRenderBridge", () => {
  it("carries what the sectors say to the renderer", () => {
    const renderer = mockRenderer();
    const sectors = mockSource<TLevelSectorListener>();

    new LevelRenderBridge(renderer, { sectors: sectors.source as ILevelSectorSource, textures: null });

    for (const listener of sectors.listeners) {
      listener({ delivered: [], released: [7] });
    }

    expect(renderer.delivered).toEqual([{ delivered: [], released: [7] }]);
  });

  it("carries what the textures say to the renderer", () => {
    const renderer = mockRenderer();
    const textures = mockSource<TLevelTextureSupplyListener>();

    new LevelRenderBridge(renderer, { sectors: null, textures: textures.source as ILevelTextureSupply });

    for (const listener of textures.listeners) {
      listener({ delivered: [], retained: null });
    }

    expect(renderer.supplied).toEqual([{ delivered: [], retained: null }]);
  });

  // A bridge outliving its level would keep feeding a renderer that has gone, which is the leak the whole
  // subscribe-and-return-a-stop shape exists to prevent.
  it("stops carrying when it is disposed", () => {
    const renderer = mockRenderer();
    const sectors = mockSource<TLevelSectorListener>();
    const textures = mockSource<TLevelTextureSupplyListener>();

    const bridge: LevelRenderBridge = new LevelRenderBridge(renderer, {
      sectors: sectors.source as ILevelSectorSource,
      textures: textures.source as ILevelTextureSupply,
    });

    bridge.dispose();

    expect(sectors.listeners.size).toBe(0);
    expect(textures.listeners.size).toBe(0);
  });

  // Disposing the bridge is not disposing the renderer: whoever made it owns it, and a level closing is not a
  // viewport closing.
  it("leaves the renderer to whoever made it", () => {
    const renderer = mockRenderer();

    new LevelRenderBridge(renderer, { sectors: null, textures: null }).dispose();

    expect(renderer.dispose).not.toHaveBeenCalled();
  });

  it("carries nothing before a level has any", () => {
    const renderer = mockRenderer();

    new LevelRenderBridge(renderer, { sectors: null, textures: null });

    expect(renderer.delivered).toEqual([]);
    expect(renderer.supplied).toEqual([]);
  });
});
