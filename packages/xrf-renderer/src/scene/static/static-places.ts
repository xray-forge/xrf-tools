import { Nullable } from "@xrf/types";
import { Matrix4 } from "three/webgpu";

import {
  IRendererInstances,
  RENDERER_FLOATS_PER_INSTANCE,
  RENDERER_HEMI_FLOATS_PER_INSTANCE,
} from "#/contract/scene/renderer-instances";
import { packStaticHemiCube } from "#/scene/static/static-hemi-cube";
import { StaticRunPool } from "#/scene/static/static-run-pool";
import { STATIC_PLACE_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";

/** Floats one place takes in the places buffer. */
const FLOATS_PER_PLACE: number = STATIC_PLACE_COLUMNS * 4;

/** Where a place's hemisphere cube starts: its sixth column. */
const HEMI_CUBE_OFFSET: number = 20;

/**
 * Where static draws stand: a matrix, hemisphere terms, an impostor, the greatest scale and a hemisphere cube each, a
 * single draw's one and an instanced draw's one an instance. Handed out in runs, uploaded as one span.
 */
export class StaticPlaces extends StaticRunPool {
  private readonly matrix: Matrix4 = new Matrix4();

  public constructor(buffers: StaticDrawBuffers) {
    super(buffers, EStaticPool.PLACES);
  }

  public free(start: number, count: number): void {
    this.runs.release(start, count);
  }

  /**
   * @param start - Where an object's places start.
   * @param instances - Its places, as the consumer put them.
   * @param placement - What places every instance: the object's own matrix.
   * @param lodStart - Where the impostors its places belong to start, or null where they belong to none.
   */
  public writePlaces(
    start: number,
    instances: IRendererInstances,
    placement: Matrix4,
    lodStart: Nullable<number> = null
  ): void {
    const impostors: Nullable<Int32Array> = lodStart === null ? null : (instances.impostors?.indices ?? null);
    const count: number = instances.transforms.length / RENDERER_FLOATS_PER_INSTANCE;
    const places: Float32Array = this.buffers.places.array as Float32Array;
    const words: Uint32Array = new Uint32Array(places.buffer, places.byteOffset, places.length);

    for (let index = 0; index < count; index += 1) {
      const at: number = (start + index) * FLOATS_PER_PLACE;

      this.matrix.fromArray(instances.transforms, index * RENDERER_FLOATS_PER_INSTANCE).premultiply(placement);
      places.set(this.matrix.elements, at);
      // Instances without terms of their own leave the vertex hemi as it is.
      places[at + 16] = instances.hemi ? instances.hemi[index * RENDERER_HEMI_FLOATS_PER_INSTANCE] : 1;
      places[at + 17] = instances.hemi ? instances.hemi[index * RENDERER_HEMI_FLOATS_PER_INSTANCE + 1] : 0;
      // The impostor a place's draw is, which an impostor shader reads its facets by; -1 for none.
      places[at + 18] = impostors && lodStart !== null && impostors[index] >= 0 ? lodStart + impostors[index] : -1;
      // What a cluster's sphere, in its mesh's own space, is scaled by where the place stands it.
      places[at + 19] = this.matrix.getMaxScaleOnAxis();

      // Written as bits: the packed halves are words, which a float view would read as numbers.
      if (instances.hemiCube) {
        packStaticHemiCube(instances.hemiCube, index, words, at + HEMI_CUBE_OFFSET);
        places[at + HEMI_CUBE_OFFSET + 3] = 1;
      } else {
        places.fill(0, at + HEMI_CUBE_OFFSET, at + HEMI_CUBE_OFFSET + 4);
      }
    }

    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /**
   * @param at - A single draw's place.
   * @param matrix - Where it stands.
   */
  public writePlace(at: number, matrix: Matrix4): void {
    const places: Float32Array = this.buffers.places.array as Float32Array;
    const first: number = at * FLOATS_PER_PLACE;

    places.set(matrix.elements, first);
    places.set([1, 0, -1, matrix.getMaxScaleOnAxis(), 0, 0, 0, 0], first + 16);
    this.span.touch(at);
    this.currentVersion += 1;
  }

  public flush(): void {
    this.span.upload(this.buffers.places, FLOATS_PER_PLACE);
    this.span.clear();
  }
}
