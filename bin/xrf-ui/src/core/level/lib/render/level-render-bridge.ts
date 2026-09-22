import { Nullable } from "@xrf/types";

import { ILevelSectorSource, ILevelTextureSupply } from "@/core/level/lib/render/level-render-protocol";
import { ILevelRenderer } from "@/core/level/lib/render/level-renderer";

/** Where a level's sectors and texture files come from, which is the loader. */
export interface ILevelRenderSources {
  sectors: Nullable<ILevelSectorSource>;
  textures: Nullable<ILevelTextureSupply>;
}

/**
 * Carries a level to a renderer, and is the only thing that knows both.
 */
export class LevelRenderBridge {
  private readonly renderer: ILevelRenderer;
  private readonly unsubscribe: Array<() => void> = [];

  public constructor(renderer: ILevelRenderer, sources: ILevelRenderSources) {
    this.renderer = renderer;

    if (sources.sectors) {
      this.unsubscribe.push(sources.sectors.subscribe((change) => this.renderer.deliver(change)));
    }

    if (sources.textures) {
      this.unsubscribe.push(sources.textures.subscribe((change) => this.renderer.supply(change)));
    }
  }

  /** Stops carrying, leaving the renderer to whoever made it. */
  public dispose(): void {
    for (const stop of this.unsubscribe) {
      stop();
    }

    this.unsubscribe.length = 0;
  }
}
