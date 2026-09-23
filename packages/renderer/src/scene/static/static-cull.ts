import { WebGPURenderer } from "three/webgpu";

import { IStaticCullCounts } from "#/scene/static/static-cull-counts";
import { createStaticCullShader, IStaticCullShader } from "#/scene/static/static-cull.tsl";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { CullView } from "#/visibility/cull-view";

/**
 * Culls every static draw on the GPU against the view drawn for, again only when the view moved or a slot changed.
 */
export class StaticCull {
  private readonly buffers: StaticDrawBuffers;
  private readonly pool: StaticDrawPool;
  private readonly shader: IStaticCullShader;
  /** What the last cull read back kept, and whether a read is in flight. */
  private counts: IStaticCullCounts = { draws: 0, triangles: 0 };
  private isReading: boolean = false;
  /** The view and pool versions the last dispatch culled against. */
  private viewVersion: number = -1;
  private poolVersion: number = -1;
  private isPending: boolean = false;

  public constructor(buffers: StaticDrawBuffers, pool: StaticDrawPool) {
    this.buffers = buffers;
    this.pool = pool;
    this.shader = createStaticCullShader(buffers);
  }

  /** What the last cull read back kept, which lags the frame by the read. */
  public get kept(): IStaticCullCounts {
    return this.counts;
  }

  /**
   * @param view - The view about to be drawn.
   */
  public take(view: CullView): void {
    if (view.version === this.viewVersion && this.pool.version === this.poolVersion) {
      return;
    }

    view.planes.forEach(({ normal, constant }, index: number) =>
      this.shader.planes[index].set(normal.x, normal.y, normal.z, constant)
    );
    this.viewVersion = view.version;
    this.poolVersion = this.pool.version;
    this.isPending = true;
  }

  /**
   * Uploads what changed and culls, where anything did.
   *
   * @param renderer - The renderer drawing.
   */
  public dispatch(renderer: WebGPURenderer): void {
    if (!this.isPending) {
      return;
    }

    // The upload comes first: the arguments it writes carry an instance count the cull then writes over.
    this.pool.flush();
    (this.buffers.counts.array as Uint32Array).fill(0);
    this.buffers.counts.needsUpdate = true;
    renderer.compute(this.shader.compute);
    this.isPending = false;
  }

  /**
   * Reads back what the last cull kept, one read in flight at a time, never waited on.
   *
   * @param renderer - The renderer drawing.
   */
  public sample(renderer: WebGPURenderer): void {
    if (this.isReading) {
      return;
    }

    this.isReading = true;
    renderer
      .getArrayBufferAsync(this.buffers.counts)
      .then((buffer: ArrayBuffer) => {
        const [draws, indices] = new Uint32Array(buffer);

        this.counts = { draws, triangles: indices / 3 };
      })
      .catch(() => {})
      .finally(() => (this.isReading = false));
  }

  public dispose(): void {
    this.shader.compute.dispose();
  }
}
