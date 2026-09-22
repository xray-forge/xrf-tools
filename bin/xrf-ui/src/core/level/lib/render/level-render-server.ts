import { EMPTY_LEVEL_FLY_MOTION, ILevelFlyMotion, ILevelMotionSource } from "@/core/level/lib/camera/level-fly-motion";
import {
  ELevelRenderRequest,
  ELevelRenderResponse,
  TLevelRenderRequest,
  TLevelRenderResponse,
} from "@/core/level/lib/render/level-render-messages";
import { LevelPreviewScene } from "@/core/level/lib/scene";
import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Where a served renderer sends what it has to say. */
export type TLevelRenderReply = (response: TLevelRenderResponse) => void;

/**
 * Draws a level for somebody else's thread.
 */
export class LevelRenderServer implements ILevelMotionSource {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly reply: TLevelRenderReply;

  private scene: Nullable<LevelPreviewScene> = null;
  private target: Nullable<OffscreenRenderTarget> = null;

  /** What the page last said the person was doing, applied by whichever frame reads it next. */
  private motion: ILevelFlyMotion = EMPTY_LEVEL_FLY_MOTION;

  public constructor(reply: TLevelRenderReply) {
    this.reply = reply;
  }

  /**
   * Takes one message.
   *
   * @param request - What the page said.
   */
  public take(request: TLevelRenderRequest): void {
    switch (request.kind) {
      case ELevelRenderRequest.START:
        return this.start(request);

      case ELevelRenderRequest.RESIZE:
        return this.target?.resize(request);

      case ELevelRenderRequest.OPEN:
        return this.scene?.open(request.level);

      case ELevelRenderRequest.DELIVER:
        return this.scene?.deliver(request.change);

      case ELevelRenderRequest.SUPPLY:
        return this.scene?.supply(request.change);

      case ELevelRenderRequest.VIEW:
        return this.scene?.setView(request.view);

      case ELevelRenderRequest.MOTION:
        this.motion = request.motion;

        return;

      case ELevelRenderRequest.MEASURE:
        return this.reply({
          geometry: this.scene?.measureSurfaceGeometry() ?? new Map(),
          id: request.id,
          kind: ELevelRenderResponse.MEASURED,
        });

      case ELevelRenderRequest.DISPOSE:
        return this.dispose();
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

  private start(request: Extract<TLevelRenderRequest, { kind: ELevelRenderRequest.START }>): void {
    this.dispose();

    this.log.info("Drawing on a canvas of", request.width, "x", request.height, "at", request.pixelRatio);

    this.target = new OffscreenRenderTarget(request.canvas, request);
    this.scene = new LevelPreviewScene(this.target, {
      onCameraMoved: (point) => this.reply({ kind: ELevelRenderResponse.CAMERA, point }),
      onReport: (stats, camera) => this.reply({ camera, kind: ELevelRenderResponse.REPORT, stats }),
      onTextures: (report) => this.reply({ kind: ELevelRenderResponse.TEXTURES, report }),
    });

    this.scene.setMotion(this);
  }

  private dispose(): void {
    if (this.scene) {
      this.log.info("Releasing everything the renderer held");
    }

    this.scene?.dispose();
    this.scene = null;
    this.target = null;
    this.motion = EMPTY_LEVEL_FLY_MOTION;
  }
}
