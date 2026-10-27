import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { cancelFlows, ExclusiveFlow, TFlow } from "@xrf/flow";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { VisualMotionBake } from "@xrf/ipc/types/xrf-visual";
import { Logger } from "@xrf/logging";
import { clamp } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { CommandOperation } from "@/core/operations/lib/command-operation";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";
import { clampMotionFps, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { VisualLoadService } from "@/core/visuals/services/visual-load";

/**
 * Playing one of the open visual's motions.
 *
 * The motion is baked once to learn how long it is, and the clock runs here; the renderer bakes and poses the model by
 * itself, told only the motion's name and the frame.
 */
@Injectable()
export class VisualMotionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The posed motion, an opening of this viewer's own beside the model's: what it plays is its view, not the model. */
  public readonly posed: SessionRootResource<VisualMotionBake> = new SessionRootResource(this.log, {
    subject: "motion",
    close: visualsCommands.closeMotion,
  });

  /**
   * Names the open visual can play, listed on demand.
   *
   * Not fetched with the model: naming them means reading every animation file it references, about fifty milliseconds
   * each, and most models are opened to be looked at rather than played.
   */
  public readonly motions: CommandOperation<Array<string>> = new CommandOperation(this.log);

  private ticker: Nullable<ReturnType<typeof setInterval>> = null;

  @Observable()
  public frame: number = 0;

  @Observable()
  public isPlaying: boolean = false;

  @Observable()
  public isLooping: boolean = true;

  /**
   * Frames a second playback advances at, which starts at the rate the format samples.
   *
   * A viewing aid rather than a property of the motion: the reported duration stays the time the engine would take,
   * which the motion's own speed already scales, and playing it at another rate only changes how long looking at it
   * takes. Slowing a two second animation down is the only way to see what a foot does in three frames.
   */
  @Observable()
  public fps: number = MOTION_SAMPLE_FPS;

  /**
   * @returns Frames the posed motion holds, or zero when nothing is posed.
   */
  @Computed()
  public get frameCount(): number {
    return this.posed.value?.frameCount ?? 0;
  }

  public constructor(private readonly loadService: VisualLoadService = inject(VisualLoadService)) {}

  /** Stops the ticker when the tool goes away, so a hidden viewer is not still animating. */
  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /** Lists what the open visual can play, once. */
  @ExclusiveFlow("motions")
  public *list(): TFlow {
    const sessionId: Nullable<string> = this.loadService.sessionId;

    if (!sessionId || this.motions.value?.length) {
      return;
    }

    yield* this.motions.run({
      name: "List visual motions",
      subject: sessionId,
      command: () => visualsCommands.listMotions(sessionId).then((listed) => listed.value),
      failure: "Could not list the motions",
    });
  }

  /**
   * Poses one motion and shows it from its first frame.
   *
   * Playback carries over: a motion picked while another was playing plays, and one picked while another was paused
   * stays paused on frame zero, so comparing two motions frame by frame does not mean pausing each one again. The first
   * pick of a session plays, because picking a motion with nothing posed is asking to see it move.
   *
   * The pose on screen is kept while the next one bakes, rather than dropped. Dropping it put the model back in its bind
   * pose for the length of a read, so switching between two motions meant watching the skeleton snap flat in between -
   * the frame stays put with it, since a frame index means nothing without the bake it counts into.
   *
   * @param name - Motion to pose, as `list` reported it.
   */
  @BoundAction()
  public async open(name: string): Promise<void> {
    const resume: boolean = this.isPlaying || !this.posed.value;
    const sessionId: Nullable<string> = this.loadService.sessionId;

    if (!sessionId) {
      this.log.info("Cannot pose a motion with no visual open:", name);

      return;
    }

    this.stopTicker();
    // Cleared rather than left standing: `play` is a no-op while this says playback is already running, which would
    // leave the controls claiming to play a motion whose ticker was just stopped.
    this.isPlaying = false;

    // A pose that failed answers false and one a newer pose superseded answers nothing, so only the latest picked, once
    // it lands, shows its first frame.
    if (
      await flowResult(
        this.posed.open(name, (motionId: SessionId) => visualsCommands.openMotion(sessionId, motionId, name))
      )
    ) {
      this.showFirstFrame(resume);
    }
  }

  /** Shows one frame, stopping playback so a dragged slider is not fought over. */
  @BoundAction()
  public seek(frame: number): void {
    this.stopTicker();
    this.isPlaying = false;
    this.frame = clamp(frame, 0, Math.max(0, this.frameCount - 1));
  }

  @BoundAction()
  public play(): void {
    if (!this.posed.value || this.isPlaying) {
      return;
    }

    // Play on the last frame restarts, rather than starting a ticker whose first tick has nowhere to go: a motion that
    // ran to its end without looping is parked there, and pressing play on it means play it again.
    if (this.frame >= this.frameCount - 1) {
      this.frame = 0;
    }

    this.isPlaying = true;
    this.startTicker();
  }

  /**
   * Plays at another rate, restarting the ticker so a change is felt without pressing play again.
   *
   * @param fps - Frames a second, clamped to something a `setInterval` can actually keep up with.
   */
  @BoundAction()
  public setFps(fps: number): void {
    this.fps = clampMotionFps(fps);

    if (this.isPlaying) {
      this.startTicker();
    }
  }

  @BoundAction()
  public pause(): void {
    this.stopTicker();
    this.isPlaying = false;
  }

  @BoundAction()
  public toggleLoop(): void {
    this.isLooping = !this.isLooping;
  }

  /**
   * Drops the posed motion, letting the backend's bake go, and stops playing, which is what leaving or opening another
   * model means.
   *
   * The playback rate survives: it is a preference about looking at motions rather than a property of the one that was
   * open. The list does not, because it named the previous model's motions; whoever shows them lists again for the
   * model that replaced it.
   */
  @BoundAction()
  public clear(): void {
    cancelFlows(this);
    this.posed.release();

    this.stopTicker();
    this.isPlaying = false;
    this.frame = 0;
    this.motions.reset();
  }

  /**
   * Shows a motion just posed from its first frame, playing on where playback ran before it.
   *
   * @param resume - Whether to play.
   */
  @BoundAction()
  private showFirstFrame(resume: boolean): void {
    this.frame = 0;

    if (resume) {
      this.play();
    }
  }

  /** One frame on, wrapping or stopping at the end depending on the loop toggle. */
  @BoundAction()
  private advance(): void {
    const last: number = this.frameCount - 1;

    if (last <= 0) {
      return;
    }

    if (this.frame >= last) {
      if (!this.isLooping) {
        return this.pause();
      }

      this.frame = 0;

      return;
    }

    this.frame += 1;
  }

  /** (Re)starts the frame ticker at the current rate, replacing any ticker already running. */
  private startTicker(): void {
    this.stopTicker();

    this.ticker = setInterval(() => this.advance(), 1_000 / this.fps);
  }

  private stopTicker(): void {
    if (this.ticker !== null) {
      clearInterval(this.ticker);
      this.ticker = null;
    }
  }
}
