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
import { IOffscreenRenderSize } from "@/core/render/lib/frame/offscreen-render-target";
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

  private readonly worker: Worker;
  /** The canvas on the page, whose drawing has been given away but whose input and layout have not. */
  private readonly target: DomRenderTarget;
  private readonly events: ILevelRendererEvents;
  private readonly controls: LevelFlyControls;
  private readonly unobserve: () => void;

  /** Answers waiting on the worker, by the number they were asked with. */
  private readonly measuring: Map<number, (geometry: ReadonlyMap<number, ILevelSurfaceGeometry>) => void> = new Map();

  private measured: number = 0;
  private frameHandle: number = 0;
  /** Whether the last motion posted said anything, so a stop is posted once rather than sixty times a second. */
  private isMoving: boolean = false;

  public constructor({ target, events }: ILevelWorkerRendererOptions) {
    this.events = events;
    this.target = target;
    this.controls = new LevelFlyControls(this.target.canvas);

    this.worker = new Worker(new URL("./level-render.worker.ts", import.meta.url), { type: "module" });
    this.worker.onmessage = (event: MessageEvent<TLevelRenderResponse>): void => this.receive(event.data);
    this.worker.onerror = (event: ErrorEvent): void => this.log.error("The level render worker failed:", event.message);

    this.log.info("Started a render worker");

    this.post({
      canvas: this.target.canvas.transferControlToOffscreen(),
      kind: ELevelRenderRequest.START,
      ...this.getSize(),
    });

    this.unobserve = this.target.observe(() => this.post({ kind: ELevelRenderRequest.RESIZE, ...this.getSize() }));

    this.pump();
  }

  public open(level: Nullable<ILevelRenderLevel>): void {
    this.post({ kind: ELevelRenderRequest.OPEN, level });
  }

  public deliver(change: ILevelSectorChange): void {
    this.post({ change, kind: ELevelRenderRequest.DELIVER });
  }

  public supply(change: ILevelTextureSupplyChange): void {
    this.post({ change, kind: ELevelRenderRequest.SUPPLY });
  }

  public setView(view: ILevelRenderView): void {
    this.post({ kind: ELevelRenderRequest.VIEW, view });
  }

  public measure(): Promise<ReadonlyMap<number, ILevelSurfaceGeometry>> {
    const id: number = (this.measured += 1);

    return new Promise((resolve) => {
      this.measuring.set(id, resolve);
      this.post({ id, kind: ELevelRenderRequest.MEASURE });
    });
  }

  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.unobserve();
    this.controls.dispose();

    // Terminating reclaims the thread whole, context and all; the message is what releases a server that is
    // not on a worker, and is said first so the two are released the same way.
    this.post({ kind: ELevelRenderRequest.DISPOSE });
    this.worker.terminate();

    this.log.info("Terminated the render worker");

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
        this.post({ kind: ELevelRenderRequest.MOTION, motion });
      }
    });
  }

  private post(request: TLevelRenderRequest): void {
    this.worker.postMessage(request, listRenderTransfers(request));
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

  private getSize(): IOffscreenRenderSize {
    return { height: this.target.height, pixelRatio: this.target.pixelRatio, width: this.target.width };
  }
}
