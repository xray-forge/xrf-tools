/**
 * How far the frames run ahead of the GPU: counts the frames submitted and not yet done, and says when another may
 * start.
 */
export class RendererFramePacing {
  /** Frames the GPU may still be working on as the next starts. */
  public limit: number = Infinity;

  private readonly onReady: () => void;
  private inFlight: number = 0;

  /**
   * @param onReady - Called once a frame is done, when another may start.
   */
  public constructor(onReady: () => void) {
    this.onReady = onReady;
  }

  /** Whether another frame may start. */
  public get isReady(): boolean {
    return this.inFlight < this.limit;
  }

  /**
   * @param done - Settles once the GPU has done everything submitted so far, the frame's work with it.
   */
  public submitted(done: Promise<void>): void {
    this.inFlight += 1;

    // A lost device settles its work too, one way or the other; either way the frame no longer waits on it.
    void done
      .catch(() => undefined)
      .then(() => {
        this.inFlight -= 1;
        this.onReady();
      });
  }
}
