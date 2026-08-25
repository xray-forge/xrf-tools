import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";

import { visualsCommands } from "@/core/bindings/commands/visuals";
import { visualsRawCommands } from "@/core/bindings/commands/visuals-raw";
import { VisualMotionBake } from "@/core/bindings/types/xrf-visual";
import { transformError } from "@/core/error/lib";
import { clampMotionFps, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { Logger } from "@/lib/logging";
import { Nullable, Optional } from "@/lib/types/general";

/** How far a motion got towards being playable, which is what a clip of it can report. */
export enum ESequenceMotionState {
  BAKING = "baking",
  READY = "ready",
  UNAVAILABLE = "unavailable",
}

/** One motion of the open visual as a sequence can use it: its baked frames, or why it has none. */
export interface ISequenceMotion {
  state: ESequenceMotionState;
  /** Why the motion cannot play, when it cannot. */
  reason: Nullable<string>;
  /** What the backend said the bake is, once it is baked. */
  bake: Nullable<VisualMotionBake>;
  /** Every frame's bone transforms, once they are read. */
  transforms: Nullable<Float32Array>;
}

/** One clip of the track: a motion, at the position its author put it. */
export interface ISequenceClip {
  /** Identity of this occurrence, because one motion can appear twice and a name cannot address a position. */
  id: string;
  /** Motion name, as `list_motions` reported it. */
  motion: string;
}

/**
 * An ordered track of the open visual's motions, and playing it through.
 *
 * A clip is authoring intent - a motion name and a position - and never baked transforms, so a track outlives the
 * model being reopened and can be checked against a model that no longer has one of its motions. What each motion
 * became is held beside the track, keyed by name: two clips of one motion share one bake rather than fetching it twice.
 *
 * Playback cuts at every boundary. Blending is a separate design with its own rules, and fading between two poses here
 * would be showing a transition the engine never performs.
 */
@Injectable()
export class VisualSequenceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private ticker: Nullable<ReturnType<typeof setInterval>> = null;

  /**
   * Bakes run one at a time, chained onto this.
   */
  private baking: Promise<void> = Promise.resolve();

  /**
   * Which track the bakes in flight belong to.
   *
   * A bake resolving after the track was cleared - or after another model was opened - belongs to a skeleton that is
   * no longer on screen, so it is dropped rather than posed against whatever replaced it.
   */
  private generation: number = 0;

  /** Names clips, so a motion added twice is two addressable clips rather than one ambiguous name. */
  private lastClipId: number = 0;

  @Observable()
  public clips: ReadonlyArray<ISequenceClip> = [];

  /** What became of each motion the track names, by motion name. */
  @Observable()
  public motions: ReadonlyMap<string, ISequenceMotion> = new Map();

  /** Which clip is playing, as an index into `clips`. */
  @Observable()
  public clipIndex: number = 0;

  /** Which frame of that clip is shown. */
  @Observable()
  public frame: number = 0;

  @Observable()
  public isPlaying: boolean = false;

  @Observable()
  public isLooping: boolean = true;

  /**
   * Frames a second playback advances at, which starts at the rate the format samples.
   *
   * A viewing aid rather than a property of the sequence: an author slowing a track down to look at a cut is not
   * changing what the track is.
   */
  @Observable()
  public fps: number = MOTION_SAMPLE_FPS;

  /**
   * @returns The clip playback is on, or null when the track is empty.
   */
  @Computed()
  public get clip(): Nullable<ISequenceClip> {
    return this.clips[this.clipIndex] ?? null;
  }

  /**
   * @returns What the playing clip's motion became, or null when there is no clip to play.
   */
  @Computed()
  public get playing(): Nullable<ISequenceMotion> {
    const clip: Nullable<ISequenceClip> = this.clip;

    return clip ? (this.motions.get(clip.motion) ?? null) : null;
  }

  /**
   * @returns The bone transforms posing the model right now, or null while the playing clip has none.
   */
  @Computed()
  public get transforms(): Nullable<Float32Array> {
    return this.playing?.transforms ?? null;
  }

  /**
   * @returns Frames the playing clip holds, or zero when nothing is playable.
   */
  @Computed()
  public get frameCount(): number {
    return this.playing?.bake?.frameCount ?? 0;
  }

  /**
   * @returns Floats one bone occupies in the playing buffer, which the scene needs to index into it.
   */
  @Computed()
  public get floatsPerBone(): number {
    return this.playing?.bake?.floatsPerBone ?? 0;
  }

  /**
   * @returns How many of the track's clips can actually play.
   */
  @Computed()
  public get playableCount(): number {
    return this.clips.filter((clip: ISequenceClip) => this.isPlayable(clip)).length;
  }

  /**
   * @returns How long the whole track runs at the rate the format samples, in seconds.
   *
   * Reported at the sample rate rather than the current one, for the same reason one motion's duration is: the
   * playback rate says how fast it is being looked at, not how long it is.
   */
  @Computed()
  public get duration(): number {
    return this.clips.reduce(
      (total: number, clip: ISequenceClip) => total + (this.motions.get(clip.motion)?.bake?.duration ?? 0),
      0
    );
  }

  /** Stops the ticker when the application goes away, so a hidden sequencer is not still animating. */
  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Appends a clip and starts baking its motion.
   *
   * Baked on being added rather than on being reached, because a bake takes long enough to be a stall mid-track and an
   * author adding a clip has already said they want to see it.
   *
   * @param motion - Motion name, as the model's motion list reported it.
   */
  @BoundAction()
  public add(motion: string): void {
    this.lastClipId += 1;

    this.clips = [...this.clips, { id: `clip-${this.lastClipId}`, motion }];

    void this.bake(motion);
  }

  /**
   * Drops one clip, keeping playback on whatever it was playing when that clip is still in the track.
   *
   * @param id - Clip to remove.
   */
  @BoundAction()
  public remove(id: string): void {
    const playing: Nullable<ISequenceClip> = this.clip;
    const clips: ReadonlyArray<ISequenceClip> = this.clips.filter((clip: ISequenceClip) => clip.id !== id);

    if (clips.length === this.clips.length) {
      return;
    }

    this.clips = clips;

    // What that clip's motion baked into stays where it is: another clip may still name it, and adding it again should
    // not mean fetching it again.
    this.follow(playing?.id === id ? null : (playing?.id ?? null));
  }

  /**
   * Moves one clip along the track, which is the ordering control a cut-only sequence needs.
   *
   * @param id - Clip to move.
   * @param offset - How many positions to move it by, negative towards the start.
   */
  @BoundAction()
  public move(id: string, offset: number): void {
    const from: number = this.clips.findIndex((clip: ISequenceClip) => clip.id === id);

    if (from < 0) {
      return;
    }

    const to: number = Math.max(0, Math.min(from + offset, this.clips.length - 1));

    if (to === from) {
      return;
    }

    const playing: Nullable<ISequenceClip> = this.clip;
    const clips: Array<ISequenceClip> = [...this.clips];
    const [moved] = clips.splice(from, 1);

    clips.splice(to, 0, moved);

    this.clips = clips;

    // Reordering is not a seek: whatever was playing goes on playing from the frame it was on, wherever in the track it
    // now sits.
    this.follow(playing?.id ?? null);
  }

  /**
   * Shows one frame of one clip, stopping playback so a dragged slider is not fought over.
   *
   * @param clipIndex - Clip to show, as an index into the track.
   * @param frame - Frame of that clip to show.
   */
  @BoundAction()
  public seek(clipIndex: number, frame: number): void {
    this.stopTicker();

    this.isPlaying = false;
    this.clipIndex = Math.max(0, Math.min(clipIndex, Math.max(0, this.clips.length - 1)));
    this.frame = Math.max(0, Math.min(frame, Math.max(0, this.frameCount - 1)));
  }

  @BoundAction()
  public play(): void {
    if (this.isPlaying || !this.playableCount) {
      return;
    }

    // A track parked on its last frame restarts, the way a single motion does: pressing play on something that already
    // ran to its end means play it again.
    if (this.isFinished()) {
      this.clipIndex = this.findPlayable(0, 1) ?? 0;
      this.frame = 0;
    }

    // A clip whose motion never baked cannot be posed, so playback starts from the next one that can.
    if (!this.isPlayable(this.clip)) {
      this.clipIndex = this.findPlayable(this.clipIndex, 1) ?? this.clipIndex;
      this.frame = 0;
    }

    this.isPlaying = true;

    this.startTicker();
  }

  @BoundAction()
  public pause(): void {
    this.stopTicker();

    this.isPlaying = false;
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
  public toggleLoop(): void {
    this.isLooping = !this.isLooping;
  }

  /**
   * Drops the track and everything baked for it, which is what opening another model means.
   *
   * The playback rate and the loop toggle survive, because they are preferences about looking at sequences rather than
   * part of the one being dropped.
   */
  @BoundAction()
  public clear(): void {
    this.generation += 1;

    this.stopTicker();

    this.isPlaying = false;
    this.clipIndex = 0;
    this.frame = 0;
    this.clips = [];
    this.motions = new Map();
  }

  /**
   * @param clip - Clip to judge, or null.
   * @returns Whether this clip has frames to pose the model with.
   */
  public isPlayable(clip: Nullable<ISequenceClip>): boolean {
    const motion: Optional<ISequenceMotion> = clip ? this.motions.get(clip.motion) : undefined;

    return Boolean(motion?.transforms && (motion.bake?.frameCount ?? 0) > 0);
  }

  /**
   * Fetches one motion's frames, once per motion and one at a time.
   *
   * @param motion - Motion name to bake.
   */
  private async bake(motion: string): Promise<void> {
    if (this.motions.has(motion)) {
      return;
    }

    const generation: number = this.generation;

    this.setMotion(motion, { bake: null, reason: null, state: ESequenceMotionState.BAKING, transforms: null });

    this.baking = this.baking.then(async () => {
      if (generation !== this.generation) {
        return;
      }

      try {
        const bake: VisualMotionBake = await visualsCommands.openMotion(motion);
        const bytes: ArrayBuffer = await visualsRawCommands.readMotion(motion);
        const expected: number = bake.frameCount * bake.boneCount * bake.floatsPerBone * Float32Array.BYTES_PER_ELEMENT;

        if (bytes.byteLength !== expected) {
          throw new Error(
            `Motion '${motion}' returned ${bytes.byteLength} bytes for ${bake.frameCount} frames of ` +
              `${bake.boneCount} bones, which needs ${expected}. The pose and its bytes came from different reads.`
          );
        }

        if (generation === this.generation) {
          this.setMotion(motion, {
            bake,
            reason: null,
            state: ESequenceMotionState.READY,
            transforms: new Float32Array(bytes),
          });
        }
      } catch (error: unknown) {
        const transformed: Error = transformError(error);

        this.log.error(`Failed to bake motion '${motion}':`, transformed);

        if (generation === this.generation) {
          this.setMotion(motion, {
            bake: null,
            reason: transformed.message,
            state: ESequenceMotionState.UNAVAILABLE,
            transforms: null,
          });
        }
      }
    });

    await this.baking;
  }

  /**
   * Records what one motion became, replacing the map so whatever reads it sees the change.
   *
   * @param motion - Motion the outcome belongs to.
   * @param state - What it became.
   */
  private setMotion(motion: string, state: ISequenceMotion): void {
    runInAction(() => {
      this.motions = new Map(this.motions).set(motion, state);
    });
  }

  /**
   * Points playback at a clip by identity, after the track around it changed.
   *
   * @param id - Clip to stay on, or null to stay at the same position in the track.
   */
  private follow(id: Nullable<string>): void {
    const at: number = id === null ? this.clipIndex : this.clips.findIndex((clip: ISequenceClip) => clip.id === id);

    this.clipIndex = Math.max(0, Math.min(at < 0 ? this.clipIndex : at, Math.max(0, this.clips.length - 1)));
    this.frame = Math.max(0, Math.min(this.frame, Math.max(0, this.frameCount - 1)));

    if (!this.clips.length) {
      this.pause();
    }
  }

  /**
   * @returns Whether playback is parked on the last frame of the last clip that can play.
   */
  private isFinished(): boolean {
    return this.frame >= this.frameCount - 1 && this.findPlayable(this.clipIndex + 1, 1) === null;
  }

  /**
   * Finds the next clip with frames to play, in one direction.
   *
   * @param from - Index to start looking at, inclusive.
   * @param step - Which way to walk the track.
   * @returns That clip's index, or null when the track holds none that way.
   */
  private findPlayable(from: number, step: number): Nullable<number> {
    for (let at: number = from; at >= 0 && at < this.clips.length; at += step) {
      if (this.isPlayable(this.clips[at])) {
        return at;
      }
    }

    return null;
  }

  /**
   * One frame on, cutting to the next playable clip at a boundary.
   *
   * A clip whose motion could not be baked is passed over rather than stalling the track on a frame it cannot show.
   * The track itself says which clips those are, so nothing is skipped silently.
   */
  @BoundAction()
  private advance(): void {
    if (this.frame < this.frameCount - 1) {
      this.frame += 1;

      return;
    }

    const next: Nullable<number> = this.findPlayable(this.clipIndex + 1, 1);

    if (next !== null) {
      this.clipIndex = next;
      this.frame = 0;

      return;
    }

    if (!this.isLooping) {
      return this.pause();
    }

    this.clipIndex = this.findPlayable(0, 1) ?? this.clipIndex;
    this.frame = 0;
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
