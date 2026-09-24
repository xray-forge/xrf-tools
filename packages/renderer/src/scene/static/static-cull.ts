import { BufferAttribute, PerspectiveCamera, Scene, Texture, WebGPURenderer } from "three/webgpu";

import { destroyStorageAttribute } from "#/internals/renderer-backend";
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
 * Its shaders are built again whenever the buffers grow, and dispatched only as far as slots and rows are used.
 */
export class StaticCull {
  private readonly buffers: StaticDrawBuffers;
  private readonly pool: StaticDrawPool;
  private readonly places: StaticPlaces;
  private shader: IStaticCullShader;
  /** The buffers' layout the shaders were built over. */
  private layout: number;
  /** Buffers the last growth replaced, freed a frame later, once no recording binds them. */
  private retiring: Array<BufferAttribute> = [];
  private readonly pyramid: StaticDepthPyramid;
  /** What the second phase draws, which the G-buffer draws after the second cull. */
  private readonly late: Scene;
  /** What the last cull read back kept, and whether a read is in flight. */
  private counts: IStaticCullCounts = {
    draws: 0,
    occludedDraws: 0,
    occludedInstances: 0,
    occludedTriangles: 0,
    triangles: 0,
  };
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
    this.layout = buffers.layout;
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
    this.retire(renderer);

    if (!this.isPending) {
      return;
    }

    this.build();
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
      // A pyramid grown for a larger drawing is a buffer the culls built before it do not read.
      this.build();
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
        const [draws, indices, occludedDraws, occludedInstances, occludedIndices] = new Uint32Array(buffer);

        this.counts = {
          draws,
          occludedDraws,
          occludedInstances,
          occludedTriangles: occludedIndices / 3,
          triangles: indices / 3,
        };
      })
      .catch(() => {})
      .finally(() => (this.isReading = false));
  }

  public dispose(): void {
    [...this.shader.early, ...this.shader.late].forEach((compute) => compute.dispose());
    this.pyramid.dispose();
  }

  /** Builds the shaders again over buffers that grew, and sizes their dispatches to what is in use. */
  private build(): void {
    if (this.layout !== this.buffers.layout) {
      const planes = this.shader.planes;

      [...this.shader.early, ...this.shader.late].forEach((compute) => compute.dispose());
      this.shader = createStaticCullShader(this.buffers);
      this.shader.planes.forEach((plane, index: number) => plane.copy(planes[index]));
      this.layout = this.buffers.layout;
    }

    // An invocation a slot handed out and a row up to the last run: nothing past them is in use.
    const slots: number = Math.max(this.pool.extent, 1);
    const rows: number = Math.max(this.places.rowExtent, 1);

    this.shader.early[0].count = slots;
    this.shader.late[0].count = slots;
    this.shader.early[1].count = rows;
    this.shader.late[1].count = rows;
  }

  /** Frees the GPU buffers a growth replaced a frame ago, and holds the ones replaced since for the next frame. */
  private retire(renderer: WebGPURenderer): void {
    this.retiring.forEach((attribute: BufferAttribute) => destroyStorageAttribute(renderer, attribute));
    this.retiring = this.buffers.takeRetired();
  }
}
