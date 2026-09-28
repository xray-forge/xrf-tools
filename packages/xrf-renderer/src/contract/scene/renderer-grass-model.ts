import { IRendererSurface } from "#/contract/scene/renderer-surface";

/**
 * One detail model of a level's library, which the grass plants.
 */
export interface IRendererGrassModel {
  /** Three floats a vertex, in renderer space. */
  positions: Float32Array;
  /** Two floats a vertex. */
  uvs: Float32Array;
  indices: Uint16Array;
  /** What it is dressed with: its base texture, cut out at its reference. */
  surface: IRendererSurface;
  /** Whether the wind moves it. */
  isWaving: boolean;
  /** The scale range it is planted at, as the library states it. */
  minScale: number;
  maxScale: number;
  /** Its bounding box's height, which a vertex's share of the sway is measured against. */
  height: number;
  /** The radius of the sphere around its bounding box. */
  radius: number;
}
