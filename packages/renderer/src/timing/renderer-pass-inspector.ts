import { Nullable } from "@xrf/types";
import { InspectorBase } from "three/webgpu";

/** Renders kept waiting for a timing before they are given up on, so a device that never resolves holds no memory. */
const PENDING_LIMIT: number = 4096;

/**
 * Tags every render three times with the pass that issued it.
 *
 * Three calls `beginRender` with the uid it later resolves a GPU duration under; the frame names the pass around it.
 */
export class RendererPassInspector extends InspectorBase {
  private current: Nullable<string> = null;
  private readonly pending: Map<string, string> = new Map();

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

  public override beginRender(uid: string): void {
    if (this.current === null) {
      return;
    }

    if (this.pending.size >= PENDING_LIMIT) {
      this.pending.delete(this.pending.keys().next().value as string);
    }

    this.pending.set(uid, this.current);
  }

  /**
   * @returns Every render still waiting for its timing, by uid.
   */
  public get issued(): ReadonlyMap<string, string> {
    return this.pending;
  }

  /**
   * @param uids - Renders whose timing has been read.
   */
  public consume(uids: ReadonlyArray<string>): void {
    for (const uid of uids) {
      this.pending.delete(uid);
    }
  }
}
