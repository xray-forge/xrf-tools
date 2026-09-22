import { ILevelFlyMotion } from "@/core/level/lib/camera/level-fly-motion";
import {
  ELevelRenderRequest,
  ELevelRenderResponse,
  listRenderTransfers,
  TLevelRenderRequest,
  TLevelRenderResponse,
} from "@/core/level/lib/render/level-render-messages";
import { ILevelSectorChange, ILevelTextureSupplyChange } from "@/core/level/lib/render/level-render-protocol";
import {
  ILevelRenderer,
  ILevelRendererEvents,
  ILevelRenderLevel,
  ILevelRenderView,
} from "@/core/level/lib/render/level-renderer";
import { LevelFlyControls } from "@/core/level/lib/scene";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { RenderWorkerClient } from "@/core/render/lib/worker/render-worker-client";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

/** What a renderer on another thread needs: somewhere to draw, and somewhere to report to. */
export interface ILevelWorkerRendererOptions {
  target: DomRenderTarget;
  events: ILevelRendererEvents;
}

/**
 * Draws the level on a thread of its own.
 */
export class LevelWorkerRenderer implements ILevelRenderer {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly client: RenderWorkerClient<TLevelRenderRequest, TLevelRenderResponse>;
  private readonly events: ILevelRendererEvents;
  private readonly controls: LevelFlyControls;

  /** Answers waiting on the worker, by the number they were asked with. */
  private readonly measuring: Map<number, (geometry: ReadonlyMap<number, ILevelSurfaceGeometry>) => void> = new Map();

  private measured: number = 0;
  private frameHandle: number = 0;
  /** Whether the last motion posted said anything, so a stop is posted once rather than sixty times a second. */
  private isMoving: boolean = false;

  public constructor({ target, events }: ILevelWorkerRendererOptions) {
    this.events = events;
    this.controls = new LevelFlyControls(target.canvas);
    this.client = new RenderWorkerClient({
      // Gestures are not forwarded: the controls above are already reading them, on the element itself.
      isInputForwarded: false,
      listTransfers: listRenderTransfers,
      log: this.log,
      onResponse: (response: TLevelRenderResponse): void => this.receive(response),
      target,
      worker: new Worker(new URL("./level-render.worker.ts", import.meta.url), { type: "module" }),
    });

    this.pump();
  }

  public open(level: Nullable<ILevelRenderLevel>): void {
    this.client.post({ kind: ELevelRenderRequest.OPEN, level });
  }

  public deliver(change: ILevelSectorChange): void {
    this.client.post({ change, kind: ELevelRenderRequest.DELIVER });
  }

  public supply(change: ILevelTextureSupplyChange): void {
    this.client.post({ change, kind: ELevelRenderRequest.SUPPLY });
  }

  public setView(view: ILevelRenderView): void {
    this.client.post({ kind: ELevelRenderRequest.VIEW, view });
  }

  public measure(): Promise<ReadonlyMap<number, ILevelSurfaceGeometry>> {
    const id: number = (this.measured += 1);

    return new Promise((resolve) => {
      this.measuring.set(id, resolve);
      this.client.post({ id, kind: ELevelRenderRequest.MEASURE });
    });
  }

  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.controls.dispose();
    this.client.dispose();

    // A measurement nobody will answer would leave whoever asked waiting for ever.
    for (const resolve of this.measuring.values()) {
      resolve(new Map());
    }

    this.measuring.clear();
  }

  /**
   * Sends what the person is doing, on the frames when they are doing anything.
   */
  private pump(): void {
    this.frameHandle = requestAnimationFrame(() => {
      this.pump();

      const motion: ILevelFlyMotion = this.controls.drain();
      const isMoving: boolean = Boolean(motion.lookX || motion.lookY || Object.values(motion.keys).some(Boolean));

      if (isMoving || this.isMoving) {
        this.isMoving = isMoving;
        this.client.post({ kind: ELevelRenderRequest.MOTION, motion });
      }
    });
  }

  private receive(response: TLevelRenderResponse): void {
    switch (response.kind) {
      case ELevelRenderResponse.CAMERA:
        return this.events.onCameraMoved(response.point);

      case ELevelRenderResponse.REPORT:
        return this.events.onReport(response.stats, response.camera);

      case ELevelRenderResponse.TEXTURES:
        return this.events.onTextures(response.report);

      case ELevelRenderResponse.MEASURED: {
        const resolve: Maybe<(geometry: ReadonlyMap<number, ILevelSurfaceGeometry>) => void> = this.measuring.get(
          response.id
        );

        this.measuring.delete(response.id);
        resolve?.(response.geometry);

        return;
      }
    }
  }
}
