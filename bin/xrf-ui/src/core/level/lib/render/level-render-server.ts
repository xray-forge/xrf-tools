import { EMPTY_LEVEL_FLY_MOTION, ILevelFlyMotion, ILevelMotionSource } from "@/core/level/lib/camera/level-fly-motion";
import {
  ELevelRenderRequest,
  ELevelRenderResponse,
  TLevelRenderRequest,
  TLevelRenderResponse,
} from "@/core/level/lib/render/level-render-messages";
import { LevelPreviewScene } from "@/core/level/lib/scene";
import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { IRenderWorkerScene, TRenderWorkerReply } from "@/core/render/lib/worker/render-worker-host";

/**
 * Draws a level for somebody else's thread.
 */
export class LevelRenderServer implements IRenderWorkerScene<TLevelRenderRequest>, ILevelMotionSource {
  private readonly scene: LevelPreviewScene;
  private readonly reply: TRenderWorkerReply<TLevelRenderResponse>;

  /** What the page last said the person was doing, applied by whichever frame reads it next. */
  private motion: ILevelFlyMotion = EMPTY_LEVEL_FLY_MOTION;

  public constructor(target: OffscreenRenderTarget, reply: TRenderWorkerReply<TLevelRenderResponse>) {
    this.reply = reply;

    this.scene = new LevelPreviewScene(target, {
      onCameraMoved: (point) => this.reply({ kind: ELevelRenderResponse.CAMERA, point }),
      onReport: (stats, camera) => this.reply({ camera, kind: ELevelRenderResponse.REPORT, stats }),
      onTextures: (report) => this.reply({ kind: ELevelRenderResponse.TEXTURES, report }),
    });

    this.scene.setMotion(this);
  }

  /**
   * Takes one message about the level.
   *
   * @param request - What the page said.
   */
  public take(request: TLevelRenderRequest): void {
    switch (request.kind) {
      case ELevelRenderRequest.OPEN:
        return this.scene.open(request.level);

      case ELevelRenderRequest.DELIVER:
        return this.scene.deliver(request.change);

      case ELevelRenderRequest.SUPPLY:
        return this.scene.supply(request.change);

      case ELevelRenderRequest.VIEW:
        return this.scene.setView(request.view);

      case ELevelRenderRequest.MOTION:
        this.motion = request.motion;

        return;

      case ELevelRenderRequest.MEASURE:
        // Whoever asked is waiting on the number they asked with, so an answer goes back either way.
        return this.reply({
          geometry: this.scene.measureSurfaceGeometry(),
          id: request.id,
          kind: ELevelRenderResponse.MEASURED,
        });
    }
  }

  /**
   * @returns What the page last said, the look forgotten as it is handed over: applying it twice would turn the
   *   camera twice as far, exactly as it would on the thread that gathered it.
   */
  public drain(): ILevelFlyMotion {
    const motion: ILevelFlyMotion = this.motion;

    this.motion = { ...motion, lookX: 0, lookY: 0 };

    return motion;
  }

  public dispose(): void {
    this.scene.dispose();
  }
}
