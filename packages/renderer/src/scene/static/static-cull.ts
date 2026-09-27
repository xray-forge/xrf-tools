import { Nullable } from "@xrf/types";
import { ComputeNode, PerspectiveCamera, Scene, Texture, Vector4, WebGPURenderer } from "three/webgpu";

import { IRendererLodSettings } from "#/contract/renderer-features";
import { IStaticCullCounts } from "#/scene/static/static-cull-counts";
import { createStaticCullShader, IStaticCullShader, IStaticViewCullShader } from "#/scene/static/static-cull.tsl";
import { StaticDepthPyramid } from "#/scene/static/static-depth-pyramid";
import { IStaticPools } from "#/scene/static/static-pools";
import {
  EStaticPool,
  STATIC_LIGHT_VIEW_START,
  STATIC_SHADOW_VIEWS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";
import { toPlaneVectors } from "#/visibility/camera-frustum";
import { CullView } from "#/visibility/cull-view";
import { IShadowFrustum } from "#/visibility/shadow-frustum";

/** Numbers a shadow view's cull is keyed by: its frustum's, the pools' and the layout's versions, then its bands'. */
const VIEW_KEY_LENGTH: number = 8;

/**
 * Culls every static draw's clusters on the GPU against the view drawn for, into every batch's region of the view's
 * list, in two phases. The first keeps what the frustum keeps and the last frame's depth does not hide; the second
 * tests what that depth hid against this frame's depth so far and keeps what it no longer hides, so nothing appears a
 * frame late. Culled again only when the view moved or anything it lists changed; what it kept stays drawn meanwhile.
 * Its shaders are built again whenever the buffers grow, and dispatched only as far as clusters, rows and batches are
 * used.
 */
export class StaticCull {
  private readonly buffers: StaticDrawBuffers;
  private readonly pools: IStaticPools;
  private shader: IStaticCullShader;
  /** The buffers' layout the shaders were built over. */
  private layout: number;
  private readonly pyramid: StaticDepthPyramid;
  /** What the second phase draws, which the G-buffer draws after the second cull. */
  private readonly late: Scene;
  /** What the last cull read back kept, and whether a read is in flight. */
  private counts: IStaticCullCounts = { clusters: 0, occludedClusters: 0, occludedTriangles: 0, triangles: 0 };
  private isReading: boolean = false;
  /** The view and pools' version the last dispatch culled against. */
  private viewVersion: number = -1;
  private poolsVersion: number = -1;
  /** What each shadow view's last cull ran against, and what this one runs against. */
  private readonly viewKeys: Array<Float64Array> = Array.from({ length: STATIC_SHADOW_VIEWS }, () =>
    new Float64Array(VIEW_KEY_LENGTH).fill(NaN)
  );
  private readonly nextKey: Float64Array = new Float64Array(VIEW_KEY_LENGTH);
  /** Which frustum owns each retained result; versions of different frustums need not be distinct. */
  private readonly viewFrustums: Array<Nullable<IShadowFrustum>> = new Array(STATIC_SHADOW_VIEWS).fill(null);
  /** Dirty views' culls, retained as one compute batch between frames. */
  private readonly pendingViews: Array<ComputeNode> = [];
  /** Whether the LOD thresholds or switch changed since the last dispatch. */
  private isLodChanged: boolean = true;
  private isPending: boolean = false;
  /** Whether this frame culled, so its depth is to be read. */
  private isCulled: boolean = false;
  /** Whether a wireframe draws, whose arguments each cull rewrites from its own. */
  private isWireframe: boolean = false;
  /** Whether what the depth hides is culled, which the frame's second phase and pyramid are there for. */
  private isOccluding: boolean = true;

  /**
   * @param buffers - What every static draw reads.
   * @param pools - What the culls read, which says when they must run again and how far.
   * @param late - The scene the second phase's batches stand in.
   */
  public constructor(buffers: StaticDrawBuffers, pools: IStaticPools, late: Scene) {
    this.buffers = buffers;
    this.pools = pools;
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
    if (!this.isLodChanged && view.version === this.viewVersion && this.pools.version === this.poolsVersion) {
      return;
    }

    toPlaneVectors(view.planes, this.shader.planes);
    this.buffers.occlusion.current.take(camera);
    this.viewVersion = view.version;
    this.poolsVersion = this.pools.version;
    this.isLodChanged = false;
    this.isPending = true;
  }

  /**
   * @param isOccluding - Whether what the depth hides is culled. Off, the first phase keeps all the frustum keeps and
   *   no second phase or pyramid runs; either way the view is culled again, since what it kept was kept by the other.
   */
  public setOccluding(isOccluding: boolean): void {
    if (isOccluding !== this.isOccluding) {
      this.isOccluding = isOccluding;
      // The first phase never occludes against a forgotten view, and only the pyramid's build takes one again.
      this.buffers.occlusion.previous.forget();
      this.isCulled = false;
      this.isPending = true;
    }
  }

  /**
   * @param isWireframe - Whether the batches' edges draw, by arguments each cull rewrites from its own.
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
    if (!this.isPending) {
      return;
    }

    this.pools.flush();
    this.build();
    (this.buffers.counts.array as Uint32Array).fill(0);
    this.buffers.counts.needsUpdate = true;
    renderer.compute([...this.shader.early]);

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
   * A shadow view's cull, its casters from every static draw its frustum reaches: run again only when the frustum
   * moved, anything it lists changed or the main camera's detail selection changed.
   *
   * @param renderer - The renderer drawing.
   * @param view - The shadow view, from zero: a sun cascade's or a local-light face slot.
   * @param cascade - Its frustum, fitted for this frame.
   * @returns Whether it culled, and what the cascade draws may have changed.
   */
  public cullView(renderer: WebGPURenderer, view: number, cascade: IShadowFrustum): boolean {
    this.pools.flush();
    this.build();

    const shader: Nullable<IStaticViewCullShader> = this.prepareView(view, cascade);

    if (!shader) {
      return false;
    }

    renderer.compute([...shader.cull]);

    return true;
  }

  /**
   * Culls a batch of shadow frustums in one submission. Each owns its view's arguments and list until its slot is
   * assigned another frustum. Unchanged results stay available without another dispatch.
   *
   * @param renderer - The renderer drawing.
   * @param firstView - The first of the consecutive shadow-view slots reserved for this batch.
   * @param frustums - The frustums in slot order, at most the number of reserved slots.
   */
  public cullViews(renderer: WebGPURenderer, firstView: number, frustums: ReadonlyArray<IShadowFrustum>): void {
    if (firstView < 0 || firstView + frustums.length > this.shader.views.length) {
      throw new RangeError("Shadow cull batch exceeds its view slots");
    }

    if (!frustums.length) {
      return;
    }

    this.pools.flush();
    this.build();
    this.pendingViews.length = 0;

    frustums.forEach((frustum: IShadowFrustum, index: number) => {
      const shader: Nullable<IStaticViewCullShader> = this.prepareView(firstView + index, frustum);

      if (shader) {
        this.pendingViews.push(...shader.cull);
      }
    });

    if (this.pendingViews.length) {
      renderer.compute(this.pendingViews);
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
      // The first depth taken since none was is culled against at once, not only once the view next moves.
      this.isPending ||= !occlusion.previous.isTaken.value;
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
        const [clusters, triangles, occludedClusters, occludedTriangles] = new Uint32Array(buffer);

        this.counts = { clusters, occludedClusters, occludedTriangles, triangles };
      })
      .catch(() => {})
      .finally(() => (this.isReading = false));
  }

  public dispose(): void {
    this.disposeShader();
    this.pendingViews.length = 0;
    this.viewFrustums.fill(null);
    this.pyramid.dispose();
  }

  /** Takes a slot for this frustum, copying its planes only where its retained result is no longer current. */
  private prepareView(view: number, frustum: IShadowFrustum): Nullable<IStaticViewCullShader> {
    const { lod } = this.buffers;
    const key: Float64Array = this.nextKey;
    // A cascade casts a progressive tree at the band the camera picks for it; a light's face casts the finest.
    const isBanded: boolean = view < STATIC_LIGHT_VIEW_START;

    key[0] = frustum.version;
    key[1] = this.pools.version;
    key[2] = this.layout;
    key[3] = isBanded ? lod.glodStart.value : 0;
    key[4] = isBanded ? lod.glodEnd.value : 0;
    key[5] = isBanded ? lod.camera.value.x : 0;
    key[6] = isBanded ? lod.camera.value.y : 0;
    key[7] = isBanded ? lod.camera.value.z : 0;

    if (
      this.viewFrustums[view] === frustum &&
      key.every((value: number, at: number) => value === this.viewKeys[view][at])
    ) {
      return null;
    }

    const shader: IStaticViewCullShader = this.shader.views[view];

    this.viewFrustums[view] = frustum;
    this.viewKeys[view].set(key);
    shader.planes.forEach((plane: Vector4, index: number) => plane.copy(frustum.planes[index]));

    return shader;
  }

  private disposeShader(): void {
    [
      ...this.shader.early,
      this.shader.late,
      ...this.shader.wire,
      ...this.shader.views.flatMap((view) => view.cull),
    ].forEach((compute: ComputeNode) => compute.dispose());
  }

  /** Builds the shaders again over buffers that grew, and sizes their dispatches to what is in use. */
  private build(): void {
    if (this.layout !== this.buffers.layout) {
      const planes: ReadonlyArray<Vector4> = this.shader.planes;

      this.disposeShader();
      // Every view culls again against the buffers as they are laid out now.
      this.viewKeys.forEach((it: Float64Array) => it.fill(NaN));
      this.shader = createStaticCullShader(this.buffers);
      this.shader.planes.forEach((plane, index: number) => plane.copy(planes[index]));
      this.layout = this.buffers.layout;
    }

    // An invocation a cluster, a row, a batch and an impostor up to the last run handed out: nothing past them is used.
    const clusters: number = Math.max(this.pools.clusterExtent, 1);
    const rows: number = Math.max(this.pools.rowExtent, 1);
    const batches: number = Math.max(this.pools.batchExtent, 1);
    const [clearEarly, clearLate, lods, singles, instanced] = this.shader.early;

    clearEarly.count = batches;
    clearLate.count = batches;
    lods.count = Math.max(this.pools.lodExtent, 1);
    singles.count = clusters;
    instanced.count = rows;
    this.shader.late.count = Math.max(this.buffers.capacity(EStaticPool.SURFACE_LIST), 1);
    this.shader.wire.forEach((compute: ComputeNode) => (compute.count = batches));

    for (const { cull } of this.shader.views) {
      const [clear, viewSingles, viewInstanced] = cull;

      clear.count = batches;
      viewSingles.count = clusters;
      viewInstanced.count = rows;
    }
  }
}
