import { BufferAttribute, PerspectiveCamera, Scene, Texture, Vector4, WebGPURenderer } from "three/webgpu";

import { IRendererLodSettings, RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-features";
import { destroyStorageAttribute } from "#/internals/renderer-backend";
import { IStaticCullCounts } from "#/scene/static/static-cull-counts";
import { createStaticCullShader, IStaticCullShader, IStaticViewCullShader } from "#/scene/static/static-cull.tsl";
import { StaticDepthPyramid } from "#/scene/static/static-depth-pyramid";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticLods } from "#/scene/static/static-lods";
import { StaticPlaces } from "#/scene/static/static-places";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { CullView } from "#/visibility/cull-view";
import { SunCascade } from "#/visibility/sun-cascade";

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
  private readonly lods: StaticLods;
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
  private lodsVersion: number = -1;
  /** Each cascade's box, pool, places and LOD versions its last cull ran against, joined. */
  private readonly viewVersions: Array<string | number> = new Array(RENDERER_MAX_SHADOW_CASCADES).fill(-1);
  /** Whether the LOD thresholds or switch changed since the last dispatch. */
  private isLodChanged: boolean = true;
  private isPending: boolean = false;
  /** Whether this frame culled, so its depth is to be read. */
  private isCulled: boolean = false;
  /** Whether a wireframe draws, whose arguments each cull rewrites from its own. */
  private isWireframe: boolean = false;

  /**
   * @param buffers - What every static draw reads.
   * @param pool - The slots.
   * @param places - The instanced draws' places and rows.
   * @param lods - The impostors of clumps of trees.
   * @param late - The scene the second phase's batches stand in.
   */
  public constructor(
    buffers: StaticDrawBuffers,
    pool: StaticDrawPool,
    places: StaticPlaces,
    lods: StaticLods,
    late: Scene
  ) {
    this.buffers = buffers;
    this.pool = pool;
    this.places = places;
    this.lods = lods;
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
   * @param settings - When a clump of trees draws as its impostor.
   * @param width - The drawing's width, in pixels, which the thresholds scale with.
   * @param height - Its height.
   * @param camera - The camera drawing it, with its matrices current.
   */
  public takeLod(settings: IRendererLodSettings, width: number, height: number, camera: PerspectiveCamera): void {
    this.isLodChanged = this.buffers.lod.take(settings, width, height, camera) || this.isLodChanged;
  }

  /**
   * @param view - The view about to be drawn.
   * @param camera - Its camera, which the depth it draws is seen from.
   */
  public take(view: CullView, camera: PerspectiveCamera): void {
    if (
      !this.isLodChanged &&
      view.version === this.viewVersion &&
      this.pool.version === this.poolVersion &&
      this.places.version === this.placesVersion &&
      this.lods.version === this.lodsVersion
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
    this.lodsVersion = this.lods.version;
    this.isLodChanged = false;
    this.isPending = true;
  }

  /**
   * @param isWireframe - Whether the arenas' line indices draw, by arguments each cull rewrites from its own.
   */
  public setWireframe(isWireframe: boolean): void {
    if (isWireframe !== this.isWireframe) {
      this.isWireframe = isWireframe;
      // The wireframe's arguments are only as current as the last cull that wrote them.
      this.isPending = true;
    }
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
    this.lods.flush();
    (this.buffers.counts.array as Uint32Array).fill(0);
    this.buffers.counts.needsUpdate = true;
    // The slot cull leaves every instanced draw at no instances, for its rows to count up after it.
    renderer.compute(this.shader.early);

    if (this.isWireframe) {
      renderer.compute(this.shader.wire[0]);
    }

    this.isPending = false;
    this.isCulled = true;
  }

  /**
   * The second phase's cull, once the first has drawn into the G-buffer: its depth reduced, and what it hid culled
   * again against it.
   *
   * @param renderer - The renderer drawing.
   * @param depth - The G-buffer's depth.
   * @param width - Its width.
   * @param height - Its height.
   */
  public cullLate(renderer: WebGPURenderer, depth: Texture, width: number, height: number): void {
    if (this.isCulled) {
      this.pyramid.build(renderer, depth, width, height);
      // A pyramid grown for a larger drawing is a buffer the culls built before it do not read.
      this.build();
      renderer.compute(this.shader.late);

      if (this.isWireframe) {
        renderer.compute(this.shader.wire[1]);
      }
    }
  }

  /**
   * Draws what the second cull kept.
   *
   * @param renderer - The renderer drawing, with the G-buffer as its target.
   * @param camera - The camera drawing.
   */
  public drawLate(renderer: WebGPURenderer, camera: PerspectiveCamera): void {
    if (this.late.children.length) {
      renderer.render(this.late, camera);
    }
  }

  /**
   * A shadow cascade's cull, its casters from every static draw its box reaches: run again only when the box moved or
   * a slot, row, place or impostor changed.
   *
   * @param renderer - The renderer drawing.
   * @param view - The cascade, from zero.
   * @param cascade - Its box, fitted for this frame.
   * @returns Whether it culled, and what the cascade draws may have changed.
   */
  public cullView(renderer: WebGPURenderer, view: number, cascade: SunCascade): boolean {
    this.build();

    const version: string = [
      cascade.version,
      this.pool.version,
      this.places.version,
      this.lods.version,
      this.layout,
      this.buffers.lod.glodStart.value,
      this.buffers.lod.glodEnd.value,
    ].join();

    if (this.viewVersions[view] === version) {
      return false;
    }

    const shader: IStaticViewCullShader = this.shader.views[view];

    this.viewVersions[view] = version;
    shader.planes.forEach((plane: Vector4, index: number) => plane.copy(cascade.planes[index]));
    renderer.compute(shader.cull);

    return true;
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
    [...this.shader.early, ...this.shader.late, ...this.shader.wire].forEach((compute) => compute.dispose());
    this.pyramid.dispose();
  }

  /** Builds the shaders again over buffers that grew, and sizes their dispatches to what is in use. */
  private build(): void {
    if (this.layout !== this.buffers.layout) {
      const planes = this.shader.planes;

      [
        ...this.shader.early,
        ...this.shader.late,
        ...this.shader.wire,
        ...this.shader.views.flatMap((view) => view.cull),
      ].forEach((compute) => compute.dispose());
      // Every cascade culls again against the buffers as they are laid out now.
      this.viewVersions.fill(-1);
      this.shader = createStaticCullShader(this.buffers);
      this.shader.planes.forEach((plane, index: number) => plane.copy(planes[index]));
      this.layout = this.buffers.layout;
    }

    // An invocation a slot handed out, and a row and an impostor up to the last run: nothing past them is in use.
    const slots: number = Math.max(this.pool.extent, 1);
    const rows: number = Math.max(this.places.rowExtent, 1);
    const [earlySlots, lods, earlyRows] = this.shader.early;
    const [lateSlots, lateRows] = this.shader.late;

    earlySlots.count = slots;
    lateSlots.count = slots;
    this.shader.wire.forEach((compute) => (compute.count = slots));
    lods.count = Math.max(this.lods.extent, 1);
    earlyRows.count = rows;
    lateRows.count = rows;

    for (const { cull } of this.shader.views) {
      cull[0].count = slots;
      cull[1].count = rows;
    }
  }

  /** Frees the GPU buffers a growth replaced a frame ago, and holds the ones replaced since for the next frame. */
  private retire(renderer: WebGPURenderer): void {
    this.retiring.forEach((attribute: BufferAttribute) => destroyStorageAttribute(renderer, attribute));
    this.retiring = this.buffers.takeRetired();
  }
}
