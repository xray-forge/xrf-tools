import { IRendererInstanceImpostors } from "#/contract/scene/renderer-instance-impostors";

/** Floats one instance's transform takes. */
export const RENDERER_FLOATS_PER_INSTANCE: number = 16;

/** Floats one instance's hemisphere terms take. */
export const RENDERER_HEMI_FLOATS_PER_INSTANCE: number = 2;

/** Floats one instance's hemisphere cube takes: a face each way along each axis. */
export const RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE: number = 6;

/**
 * The places one geometry stands, drawn together.
 */
export interface IRendererInstances {
  /** Sixteen floats an instance, column major, placing it in renderer space. */
  transforms: Float32Array;
  /**
   * Two floats an instance, scaling then offsetting its vertices' hemisphere term: a tree's `c_scale.w` and
   * `c_bias.w`, as the engine binds them.
   */
  hemi?: Float32Array;
  /**
   * Six floats an instance, `+x +y +z -x -y -z` in renderer space: how much sky and light reach it from each way, which
   * a vertex takes by its world normal (`deffer_model_*.vs`) in place of its own term. A dynamic object's lighting.
   */
  hemiCube?: Float32Array;
  /**
   * The impostor each place belongs to. A tree's place is drawn only while its impostor is near enough; the places of an
   * impostor surface are the impostors themselves, drawn only while theirs is far enough.
   */
  impostors?: IRendererInstanceImpostors;
}
