import { Nullable } from "@xrf/types";

/**
 * One optional part of the frame, made for what is wanted of it and kept while its key stands: another key makes it
 * again, nothing wanted lets it go. Its generation counts the makings, for a stage reading it to be made again with it.
 */
export class FrameStage<T> {
  private readonly release: (value: T) => void;
  private current: Nullable<T> = null;
  private key: Nullable<string> = null;
  private made: number = 0;

  /**
   * @param release - Lets a value go once its key is gone.
   */
  public constructor(release: (value: T) => void) {
    this.release = release;
  }

  /** What the stage holds now. */
  public get value(): Nullable<T> {
    return this.current;
  }

  /** Bumped every time the stage makes or lets go of its value. */
  public get generation(): number {
    return this.made;
  }

  /**
   * @param wanted - What the value is made for, or null for none.
   * @param create - Makes it for what is wanted.
   * @param toKey - What of the wanted makes it again, when it changes.
   * @returns Whether the value changed.
   */
  public reconcile<W>(wanted: Nullable<W>, create: (wanted: W) => T, toKey: (wanted: W) => string = String): boolean {
    const key: Nullable<string> = wanted === null ? null : toKey(wanted);

    if (key === this.key) {
      return false;
    }

    this.clear();
    this.key = key;
    this.current = wanted === null ? null : create(wanted);
    this.made += 1;

    return true;
  }

  /** Lets the value go for good. */
  public dispose(): void {
    this.clear();
    this.key = null;
  }

  private clear(): void {
    if (this.current !== null) {
      this.release(this.current);
      this.current = null;
    }
  }
}
