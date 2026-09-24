import { Nullable } from "@xrf/types";
import { RenderTarget } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { ShadowUniforms } from "#/uniforms/shadow-uniforms";
import { SunCascade } from "#/visibility/sun-cascade";

/**
 * One cascade of the sun's shadow: its casters culled against its box, the cells it reaches shown, then drawn into
 * its map, depth alone. Drawn again only where the box moved or anything it casts from changed: a still camera over a
 * still level pays nothing for its shadow. In the frame only while shadows draw that many cascades.
 */
export class ShadowPass implements IRendererPass {
  public readonly name: string;

  private readonly view: number;
  private readonly target: RenderTarget;
  private readonly casters: IStaticShadowCasters;
  private readonly cull: StaticCull;
  private readonly shadows: ShadowUniforms;
  /** The casters' version its map was drawn at, or null before it was drawn at all. */
  private drawnVersion: Nullable<number> = null;
  /** Frames it has been in, which its staggered rate is counted by. */
  private frames: number = 0;

  /**
   * @param view - The cascade, from zero.
   * @param targets - The frame's targets, whose shadow maps it draws into.
   * @param casters - What the cascades draw.
   * @param cull - What culls the static draws, per cascade too.
   * @param shadows - The cascades, fitted for the frame.
   * @param resolution - Texels its map is across.
   */
  public constructor(
    view: number,
    targets: RendererTargets,
    casters: IStaticShadowCasters,
    cull: StaticCull,
    shadows: ShadowUniforms,
    resolution: number
  ) {
    this.name = `shadow:${view}`;
    this.view = view;
    this.target = targets.shadows[view];
    this.casters = casters;
    this.cull = cull;
    this.shadows = shadows;
    this.target.setSize(resolution, resolution);
  }

  public render({ renderer }: IRendererFrame): void {
    const cascade: SunCascade = this.shadows.cascades[this.view];

    this.frames += 1;

    if (this.drawnVersion !== null && this.shadows.isStaggered && !isShadowCascadeDue(this.view, this.frames)) {
      return;
    }

    const isCulled: boolean = this.cull.cullView(renderer, this.view, cascade);

    if (!isCulled && this.drawnVersion === this.casters.shadowVersion) {
      return;
    }

    this.drawnVersion = this.casters.shadowVersion;
    this.shadows.commit(this.view);
    this.casters.showShadowCells(this.view, cascade.planes);
    renderer.setRenderTarget(this.target);
    renderer.clear(false, true, false);
    renderer.sortObjects = false;
    renderer.render(this.casters.cascadeScenes[this.view], cascade.camera);
    renderer.sortObjects = true;
  }

  /** Gives its map back to nothing: the target stays, at a texel, so the sun's bindings never change. */
  public dispose(): void {
    this.target.setSize(1, 1);
  }
}

/**
 * @param view - A cascade.
 * @param frame - The frame, counted from one.
 * @returns Whether it may draw this frame: the nearest every frame, the second every other, the third every fourth,
 *   each on frames the ones before it leave, so at most two draw in one frame.
 */
export function isShadowCascadeDue(view: number, frame: number): boolean {
  const rate: number = 1 << view;

  return view === 0 || frame % rate === (1 << (view - 1)) % rate;
}
