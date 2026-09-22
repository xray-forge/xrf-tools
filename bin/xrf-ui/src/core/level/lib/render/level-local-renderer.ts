import { ILevelSectorChange, ILevelTextureSupplyChange } from "@/core/level/lib/render/level-render-protocol";
import {
  ILevelRenderer,
  ILevelRendererEvents,
  ILevelRenderLevel,
  ILevelRenderView,
} from "@/core/level/lib/render/level-renderer";
import { LevelFlyControls, LevelPreviewScene } from "@/core/level/lib/scene";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { Nullable } from "@/lib/types/general";

/** What a renderer needs to exist at all: somewhere to draw, and somewhere to report to. */
export interface ILevelLocalRendererOptions {
  container: HTMLElement;
  events: ILevelRendererEvents;
}

/**
 * Draws the level on the thread that asked.
 */
export class LevelLocalRenderer implements ILevelRenderer {
  private readonly scene: LevelPreviewScene;
  private readonly controls: LevelFlyControls;

  public constructor({ container, events }: ILevelLocalRendererOptions) {
    const target: DomRenderTarget = new DomRenderTarget(container);

    this.scene = new LevelPreviewScene(target, {
      onCameraMoved: (point) => events.onCameraMoved(point),
      onReport: (stats, camera) => events.onReport(stats, camera),
      onTextures: (report) => events.onTextures(report),
    });

    // On the canvas rather than the container: it is what takes focus and what the pointer is captured on.
    this.controls = new LevelFlyControls(target.canvas);
    this.scene.setMotion(this.controls);
  }

  public open(level: Nullable<ILevelRenderLevel>): void {
    this.scene.open(level);
  }

  public deliver(change: ILevelSectorChange): void {
    this.scene.deliver(change);
  }

  public supply(change: ILevelTextureSupplyChange): void {
    this.scene.supply(change);
  }

  public setView(view: ILevelRenderView): void {
    this.scene.setView(view);
  }

  public measure(): Promise<ReadonlyMap<number, ILevelSurfaceGeometry>> {
    return Promise.resolve(this.scene.measureSurfaceGeometry());
  }

  public dispose(): void {
    this.controls.dispose();
    this.scene.dispose();
  }
}
