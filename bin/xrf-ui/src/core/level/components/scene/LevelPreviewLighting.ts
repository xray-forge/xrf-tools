import { AmbientLight, Color, DirectionalLight, Object3D } from "three";

import { DEFAULT_SUN_DISTANCE, SUN_DISTANCE_MARGIN } from "@/core/level/components/scene/level-lighting-config";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toSunPosition } from "@/core/level/lib/lighting/level-sun";

/**
 * The light a level preview is drawn under, which belongs to the viewer rather than to the level.
 */
export class LevelPreviewLighting {
  private readonly parent: Object3D;
  private readonly sun: DirectionalLight = new DirectionalLight();
  private readonly ambient: AmbientLight = new AmbientLight();

  /** How far out the sun is put, which only has to clear whatever the level turns out to span. */
  private distance: number = DEFAULT_SUN_DISTANCE;
  private lighting: ILevelLighting = DEFAULT_LEVEL_LIGHTING;

  public constructor(parent: Object3D) {
    this.parent = parent;

    this.parent.add(this.ambient);
    this.parent.add(this.sun);
    this.apply(DEFAULT_LEVEL_LIGHTING);
  }

  /**
   * Takes what the viewer is lighting with.
   *
   * @param lighting - The sun and the hemisphere standing in for one.
   */
  public apply(lighting: ILevelLighting): void {
    this.lighting = lighting;

    this.sun.intensity = lighting.sunIntensity;
    this.sun.color = new Color(lighting.sunColor);
    this.ambient.intensity = lighting.ambientIntensity;
    this.ambient.color = new Color(lighting.ambientColor);

    this.place();
  }

  /**
   * Takes how far the level reaches, so the one directional light clears all of it rather than only what it was
   * framed on.
   *
   * @param radius - The level's own radius, or zero for a level that reports none.
   */
  public setReach(radius: number): void {
    this.distance = Math.max(DEFAULT_SUN_DISTANCE, radius * SUN_DISTANCE_MARGIN);

    this.place();
  }

  /** Takes the lights out of the scene. Nothing else here owns anything to dispose. */
  public dispose(): void {
    this.parent.remove(this.ambient);
    this.parent.remove(this.sun);
  }

  private place(): void {
    const [x, y, z] = toSunPosition(this.lighting, this.distance);

    this.sun.position.set(x, y, z);
  }
}
