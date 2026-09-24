import { IRendererInstanceImpostors } from "#/contract/scene/renderer-impostors";

/** Floats one instance's transform takes. */
export const RENDERER_FLOATS_PER_INSTANCE: number = 16;

/** Floats one instance's hemisphere terms take. */
export const RENDERER_HEMI_FLOATS_PER_INSTANCE: number = 2;

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
   * The impostor each place belongs to. A tree's place is drawn only while its impostor is near enough; the places of an
   * impostor surface are the impostors themselves, drawn only while theirs is far enough.
   */
  impostors?: IRendererInstanceImpostors;
}

/**
 * A geometry drawn with surfaces, where a transform or a set of instances places it.
 */
export interface IRendererObject {
  /** The key the geometry was put under. */
  geometry: string;
  /** The keys of the surfaces its groups draw with, by slot. */
  surfaces: ReadonlyArray<string>;
  /** Sixteen floats, column major, placing it in renderer space; identity when left out. */
  matrix?: ReadonlyArray<number>;
  /** The key of the skeleton it is skinned to, for a geometry carrying skin indices and weights. */
  skeleton?: string;
  /** The index range drawn, narrowing its geometry's groups: a model's detail level. */
  drawRange?: { start: number; count: number };
  /** Where it stands, for a geometry drawn in many places at once; its matrix places every one of them. */
  instances?: IRendererInstances;
}

/**
 * What of an object moves between threads: its instance arrays, never copied.
 *
 * @param object - The object about to be posted.
 * @returns Its buffers.
 */
export function listRendererObjectTransfers(object: IRendererObject): Array<Transferable> {
  return [object.instances?.transforms, object.instances?.hemi, object.instances?.impostors?.indices]
    .filter((array): array is Float32Array | Int32Array => array !== undefined)
    .map((array) => array.buffer);
}
