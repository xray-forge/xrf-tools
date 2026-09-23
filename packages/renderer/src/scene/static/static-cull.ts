import { PerspectiveCamera, Scene, Texture, WebGPURenderer } from "three/webgpu";

import { IStaticCullCounts } from "#/scene/static/static-cull-counts";
import { createStaticCullShader, IStaticCullShader } from "#/scene/static/static-cull.tsl";
import { StaticDepthPyramid } from "#/scene/static/static-depth-pyramid";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticPlaces } from "#/scene/static/static-places";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { CullView } from "#/visibility/cull-view";

/**
 * Culls every static draw on the GPU against the view drawn for, single draws by slot and instanced ones by row, in
 * two phases. The first draws what the frustum keeps and the last frame's depth does not hide; the second tests what
 * that depth hid against this frame's depth so far and draws what it no longer hides, so nothing appears a frame
 * late. Culled again only when the view moved or a slot, row or place changed; what it kept stays drawn meanwhile.
 */
export class StaticCull {
  private readonly buffers: StaticDrawBuffers;
  private readonly pool: StaticDrawPool;
  private readonly places: StaticPlaces;
  private readonly shader: IStaticCullShader;
  private readonly pyramid: StaticDepthPyramid;
  /** What the second phase draws, which the G-buffer draws after the second cull. */
  private readonly late: Scene;
  /** What the last cull read back kept, and whether a read is in flight. */
  private counts: IStaticCullCounts = { draws: 0, triangles: 0 };
  private isReading: boolean = false;
  /** The view and pool versions the last dispatch culled against. */
  private viewVersion: number = -1;
  private poolVersion: number = -1;
  private placesVersion: number = -1;
  private isPending: boolean = false;
  /** Whether this frame culled, so its depth is to be read. */
  private isCulled: boolean = false;

  /**
   * @param buffers - What every static draw reads.
   * @param pool - The slots.
   * @param places - The instanced draws' places and rows.
   * @param late - The scene the second phase's batches stand in.
   */
  public constructor(buffers: StaticDrawBuffers, pool: StaticDrawPool, places: StaticPlaces, late: Scene) {
    this.buffers = buffers;
    this.pool = pool;
    this.places = places;
    this.late = late;
    this.shader = createStaticCullShader(buffers);
    this.pyramid = new StaticDepthPyramid(buffers);
  }

  /** What the last cull read back kept, which lags the frame by the read. */
  public get kept(): IStaticCullCounts {
    return this.counts;
  }

  /**
   * @param view - The view about to be drawn.
   * @param camera - Its camera, which the depth it draws is seen from.
   */
  public take(view: CullView, camera: PerspectiveCamera): void {
    if (
      view.version === this.viewVersion &&
      this.pool.version === this.poolVersion &&
      this.places.version === this.placesVersion
    ) {
      return;
    }

    view.planes.forEach(({ normal, constant }, index: number) =>
      this.shader.planes[index].set(normal.x, normal.y, normal.z, constant)
    );
    this.buffers.occlusion.current.take(camera);
    this.viewVersion = view.version;
    this.poolVersion = this.pool.version;
    this.placesVersion = this.places.version;
    this.isPending = true;
  }

  /**
   * The first cull, before anything draws: uploads what changed and culls, where anything did.
   *
   * @param renderer - The renderer drawing.
   */
  public dispatch(renderer: WebGPURenderer): void {
    if (!this.isPending) {
      return;
    }

    // The upload comes first: the arguments it writes carry an instance count the cull then writes over.
    this.pool.flush();
    this.places.flush();
    (this.buffers.counts.array as Uint32Array).fill(0);
    this.buffers.counts.needsUpdate = true;
    // The slot cull leaves every instanced draw at no instances, for its rows to count up after it.
    renderer.compute(this.shader.early);
    this.isPending = false;
    this.isCulled = true;
  }

  /**
   * The second phase, once the first has drawn into the G-buffer: its depth reduced, what it hid culled again against
   * it, and what that keeps drawn.
   *
   * @param renderer - The renderer drawing, with the G-buffer as its target.
   * @param camera - The camera drawing.
   * @param depth - The G-buffer's depth.
   * @param width - Its width.
   * @param height - Its height.
   */
  public drawLate(
    renderer: WebGPURenderer,
    camera: PerspectiveCamera,
    depth: Texture,
    width: number,
    height: number
  ): void {
    if (this.isCulled) {
      this.pyramid.build(renderer, depth, width, height);
      renderer.compute(this.shader.late);
    }

    if (this.late.children.length) {
      renderer.render(this.late, camera);
    }
  }

  /**
   * Reduces the depth the frame finished with, for the next frame's first cull to read.
   *
   * @param renderer - The renderer drawing.
   * @param depth - The G-buffer's depth.
   * @param width - Its width.
   * @param height - Its height.
   */
  public finish(renderer: WebGPURenderer, depth: Texture, width: number, height: number): void {
    if (!this.isCulled) {
      return;
    }

    const { occlusion } = this.buffers;

    if (this.pyramid.build(renderer, depth, width, height)) {
      // Laid out for another size, a pyramid read by the last view would be read wrong.
      occlusion.previous.forget();
    } else {
      occlusion.previous.copy(occlusion.current);
    }

    this.isCulled = false;
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
    [...this.shader.early, ...this.shader.late].forEach((compute) => compute.dispose());
    this.pyramid.dispose();
  }
}
