import { Object3D, Vector3 } from "three";

import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { RenderPreviewLighting } from "@/core/render/lib/lighting/RenderPreviewLighting";

import { LevelSunMarker } from "./LevelSunMarker";

/**
 * The light a level preview is drawn under, which belongs to the viewer rather than to the level.
 */
export class LevelPreviewLighting {
  private readonly lights: RenderPreviewLighting;
  /** The sun made visible, since a light itself draws nothing and its angles are read off the surfaces alone. */
  private readonly marker: LevelSunMarker;

  public constructor(parent: Object3D) {
    this.lights = new RenderPreviewLighting(parent, DEFAULT_LEVEL_LIGHTING);
    this.marker = new LevelSunMarker(parent);

    this.apply(DEFAULT_LEVEL_LIGHTING);
  }

  /**
   * Takes what the viewer is lighting with.
   *
   * @param lighting - The sun and the hemisphere standing in for one.
   */
  public apply(lighting: ILevelLighting): void {
    this.lights.apply(lighting);
    this.marker.setSun(this.lights.direction, lighting.sunColor);
  }

  /**
   * Takes how far the level reaches, so the one directional light clears all of it rather than only what it was
   * framed on.
   *
   * @param radius - The level's own radius, or zero for a level that reports none.
   */
  public setReach(radius: number): void {
    this.lights.setReach(radius);
  }

  /**
   * Whether the sun is drawn as well as shone.
   *
   * @param isVisible - What the toolbar asks for.
   */
  public setSunVisible(isVisible: boolean): void {
    this.marker.setVisible(isVisible);
  }

  /**
   * Keeps the sun marker in the sky over wherever the camera has flown to.
   *
   * @param position - Where the camera is now.
   */
  public follow(position: Vector3): void {
    this.marker.follow(position);
  }

  /** Takes the lights and the marker out of the scene. */
  public dispose(): void {
    this.lights.dispose();
    this.marker.dispose();
  }
}
