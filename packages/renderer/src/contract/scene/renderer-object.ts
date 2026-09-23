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
  /**
   * The index range drawn, narrowing its geometry's groups: a model's detail level. Applies to the geometry itself, so
   * a geometry is narrowed for every object drawing it.
   */
  drawRange?: { start: number; count: number };
}
