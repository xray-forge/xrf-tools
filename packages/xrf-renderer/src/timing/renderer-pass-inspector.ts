import { Maybe, Nullable } from "@xrf/types";
import { InspectorBase } from "three/webgpu";

import { TIssuedRender } from "#/timing/issued-render";

/** Matches the frame a render context's timestamp uid belongs to: three spells them `<context>:f<frame>`. */
const FRAME_PATTERN: RegExp = /:f(\d+)$/;

/** Frames kept waiting for their timings, so renders three never timed are let go of rather than walked every read. */
const FRAME_LIMIT: number = 16;

/**
 * Tags every render three times with the pass that issued it, while the device is timing.
 *
 * Three calls `beginRender` with the uid it later resolves a GPU duration under; the frame names the pass around it.
 */
export class RendererPassInspector extends InspectorBase {
  /** Whether renders are tagged: nothing reads them while the device is not timing. */
  public isRecording: boolean = true;

  private current: Nullable<string> = null;
  /** The renders waiting for their timings, by the frame they were issued in, oldest first. */
  private readonly pending: Map<number, Array<TIssuedRender>> = new Map();

  /**
   * @param pass - The pass whose renders follow.
   */
  public enter(pass: string): void {
    this.current = pass;
  }

  /** Ends the pass; a render outside every pass is not attributed. */
  public leave(): void {
    this.current = null;
  }

  /**
   * @param isRecording - Whether renders are tagged from now on; the ones waiting are let go of when they are not.
   */
  public setRecording(isRecording: boolean): void {
    this.isRecording = isRecording;

    if (!isRecording) {
      this.pending.clear();
    }
  }

  public override beginCompute(uid: string): void {
    this.beginRender(uid);
  }

  public override beginRender(uid: string): void {
    if (!this.isRecording || this.current === null) {
      return;
    }

    const match: Nullable<RegExpMatchArray> = uid.match(FRAME_PATTERN);

    if (!match) {
      return;
    }

    const frame: number = Number(match[1]);
    let renders: Maybe<Array<TIssuedRender>> = this.pending.get(frame);

    if (!renders) {
      renders = [];
      this.pending.set(frame, renders);

      if (this.pending.size > FRAME_LIMIT) {
        this.pending.delete(this.pending.keys().next().value as number);
      }
    }

    renders.push([uid, this.current]);
  }

  /**
   * @returns Every render still waiting for its timing, by the frame it was issued in, oldest first.
   */
  public get issued(): ReadonlyMap<number, ReadonlyArray<TIssuedRender>> {
    return this.pending;
  }

  /**
   * @param frames - Frames whose timings have been read.
   */
  public consume(frames: ReadonlyArray<number>): void {
    for (const frame of frames) {
      this.pending.delete(frame);
    }
  }
}
