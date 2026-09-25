import { renderGroup, uniform } from "three/tsl";
import { Matrix4, PerspectiveCamera, UniformNode, Vector3, Vector4 } from "three/webgpu";

import { IRendererShadowSettings, RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-features";
import { SunCascade } from "#/visibility/sun-cascade";
import { SunViewRays } from "#/visibility/sun-view-rays";

/**
 * The sun's shadow as the frame reads it: each cascade fitted to the camera, and what sampling one takes, in three's
 * render group like every other uniform of the frame.
 */
export class ShadowUniforms {
  /** Every cascade there can be; the settings say how many draw. */
  public readonly cascades: ReadonlyArray<SunCascade> = Array.from(
    { length: RENDERER_MAX_SHADOW_CASCADES },
    () => new SunCascade()
  );
  /** Each cascade's world to shadow clip space. */
  public readonly matrices: ReadonlyArray<UniformNode<"mat4", Matrix4>> = this.cascades.map(() =>
    uniform(new Matrix4()).setGroup(renderGroup)
  );
  /** Each cascade's texel, in metres, a component a cascade. */
  public readonly texels = uniform(new Vector4()).setGroup(renderGroup);
  /** Cascades drawn: none while shadows are off. */
  public readonly count = uniform(0).setGroup(renderGroup);
  /** Texels the filter reaches each way. */
  public readonly filter = uniform(1).setGroup(renderGroup);
  /** Texels a point moves along its normal before it is compared. */
  public readonly bias = uniform(1).setGroup(renderGroup);
  /** How far in from a cascade's edge the next is mixed in, as a share of its width. */
  public readonly blend = uniform(0).setGroup(renderGroup);
  /** Texels each map is across. */
  public readonly resolution = uniform(1).setGroup(renderGroup);
  /** Where the camera looks, which the last cascade fades out towards. */
  public readonly forward = uniform(new Vector3(0, 0, -1)).setGroup(renderGroup);

  /** The view's edges, which the cascades are placed along one after another. */
  private readonly rays: SunViewRays = new SunViewRays();

  /** Cascades drawn, as the last fit took them. */
  public get drawn(): number {
    return this.count.value;
  }

  /** Whether cascades are drawn at their staggered rates. */
  public isStaggered: boolean = true;

  /**
   * Fits every cascade the settings draw to the camera, for this frame. What the sun samples a cascade by changes only
   * once its map is drawn again, by `commit`.
   *
   * @param camera - The camera drawing, its world matrix current.
   * @param direction - Where the sun's light travels.
   * @param settings - The shadow settings.
   */
  public fit(camera: PerspectiveCamera, direction: Vector3, settings: IRendererShadowSettings): void {
    const count: number = settings.isEnabled ? Math.min(settings.cascades.length, RENDERER_MAX_SHADOW_CASCADES) : 0;

    this.count.value = count;
    this.filter.value = settings.filter;
    this.bias.value = settings.bias;
    // Kept short of the map's middle, where every point would blend.
    this.blend.value = Math.min(Math.max(settings.blend, 0), 0.4);
    this.resolution.value = settings.resolution;
    this.isStaggered = settings.isStaggered;

    camera.getWorldDirection(this.forward.value);
    // Each cascade starts where the view's edges leave the one before it, the first at the near plane.
    this.rays.reset(camera);

    for (let view = 0; view < count; view += 1) {
      this.cascades[view].fit(
        camera,
        this.rays,
        direction,
        settings.cascades[view],
        settings.resolution,
        settings.reach
      );
    }
  }

  /**
   * Has the sun sample a cascade as it was fitted when its map was last drawn, which is what the map holds.
   *
   * @param view - The cascade just drawn.
   */
  public commit(view: number): void {
    const cascade: SunCascade = this.cascades[view];

    this.matrices[view].value.multiplyMatrices(cascade.camera.projectionMatrix, cascade.camera.matrixWorldInverse);
    this.texels.value.setComponent(view, cascade.texel);
  }
}
