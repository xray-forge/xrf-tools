import { Maybe } from "@xrf/types";

import { IRendererInstances } from "#/contract/scene/renderer-instances";

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
  return [object.instances?.transforms, object.instances?.hemi, object.instances?.impostors?.indices].flatMap(
    (array: Maybe<ArrayBufferView>) => (array ? [array.buffer] : [])
  );
}
