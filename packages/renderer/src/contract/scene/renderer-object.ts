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
}
