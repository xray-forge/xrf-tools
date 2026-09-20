import { AmbientLight, Color, DirectionalLight, Object3D, Vector3 } from "three";

import { IRenderLighting, toRenderSunPosition } from "@/core/render/lib/lighting/render-lighting";

/** Where the light stands for a scene that has reported no extent, in the scene's own unit. */
export const DEFAULT_LIGHT_DISTANCE: number = 500;

/** Times what it lights the light is put at, so it clears the geometry rather than standing inside it. */
export const LIGHT_DISTANCE_MARGIN: number = 2;

/**
 * The two lights every X-Ray preview is drawn under: one direction, and a fill for everything it misses.
 */
export class RenderPreviewLighting {
  private readonly parent: Object3D;
  private readonly sun: DirectionalLight = new DirectionalLight();
  private readonly ambient: AmbientLight = new AmbientLight();

  /** How far out the light is put, which only has to clear whatever it turns out to be lighting. */
  private distance: number = DEFAULT_LIGHT_DISTANCE;
  private lighting: IRenderLighting;

  public constructor(parent: Object3D, lighting: IRenderLighting) {
    this.parent = parent;
    this.lighting = lighting;

    this.parent.add(this.ambient);
    this.parent.add(this.sun);
    this.apply(lighting);
  }

  /** Where the light stands, for anything drawn along the same bearing. */
  public get direction(): Vector3 {
    return new Vector3(...toRenderSunPosition(this.lighting, 1));
  }

  /**
   * Takes what the scene is lit with.
   *
   * @param lighting - The direction, its strength and colour, and the fill.
   */
  public apply(lighting: IRenderLighting): void {
    this.lighting = lighting;

    this.sun.intensity = lighting.sunIntensity;
    this.sun.color = new Color(lighting.sunColor);
    this.ambient.intensity = lighting.ambientIntensity;
    this.ambient.color = new Color(lighting.ambientColor);

    this.place();
  }

  /**
   * Takes how far what is lit reaches, so the one directional light clears all of it.
   *
   * @param radius - The subject's own radius, or zero for one that reports none.
   */
  public setReach(radius: number): void {
    this.distance = Math.max(DEFAULT_LIGHT_DISTANCE, radius * LIGHT_DISTANCE_MARGIN);

    this.place();
  }

  /** Takes the lights out of the scene. Nothing here owns anything else to dispose. */
  public dispose(): void {
    this.parent.remove(this.ambient);
    this.parent.remove(this.sun);
  }

  private place(): void {
    const [x, y, z] = toRenderSunPosition(this.lighting, this.distance);

    this.sun.position.set(x, y, z);
  }
}
