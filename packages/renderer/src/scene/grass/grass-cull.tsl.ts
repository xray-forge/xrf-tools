import {
  atomicAdd,
  Continue,
  dot,
  float,
  Fn,
  If,
  instanceIndex,
  int,
  max,
  Return,
  select,
  sqrt,
  storage,
  uint,
  vec3,
  vec4,
} from "three/tsl";
import { ComputeNode, Node } from "three/webgpu";

import { IGrassBuffers } from "#/scene/grass/grass-buffers";
import { GRASS_CACHE_KEY_WORDS, GRASS_CACHE_VECTORS } from "#/scene/grass/grass-cache-buffers";
import { toGrassCacheItems, toGrassCacheKeys, toGrassCacheShapes } from "#/scene/grass/grass-cache.tsl";
import { GRASS_ITEM_VECTORS } from "#/scene/grass/grass-item-buffers";
import { toGrassCounts, toGrassPlanted, toGrassPlantedModels } from "#/scene/grass/grass-items.tsl";
import { IGrassRingSlot } from "#/scene/grass/grass-ring-slot";
import { isGrassCellCurrent, toGrassRingCells, toGrassRingSlot } from "#/scene/grass/grass-ring.tsl";
import { GRASS_BOX_GROWTH, isGrassOutsideView } from "#/scene/grass/grass-view.tsl";
import { loopNamed } from "#/shader/named-loop.tsl";
import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/**
 * `UpdateVisibleM`, a thread a cell of the ring: a cell not planted with the slot it holds now, outside the view or past
 * `dm_fade` passes over, and each of its tufts is shrunk by the slot's distance, dropped where too small to see,
 * stilled where too small to see sway, culled by the view, and appended.
 *
 * @param buffers - What the grass is planted from and into.
 * @param uniforms - Where the camera stands and what the planting is set to.
 * @param ring - How far the ring reaches.
 * @param discard - `r_ssaDISCARD`, the screen area below which the engine drops what it would draw.
 * @returns The pass; its dispatch is the ring's cells, set before each frame's.
 */
export function createGrassCull(
  buffers: IGrassBuffers,
  uniforms: GrassUniforms,
  ring: GrassRingUniforms,
  discard: Node<"float">
): ComputeNode {
  const { level, cache, items } = buffers;
  const models: number = level.modelCount;
  const { capacity } = items;
  const shapes = storage(level.models, "vec4", Math.max(models, 1) * 2).toReadOnly();
  const keys = toGrassCacheKeys(cache).toReadOnly();
  const cellShapes = toGrassCacheShapes(cache).toReadOnly();
  const cached = toGrassCacheItems(cache).toReadOnly();
  const counts = toGrassCounts(level);
  const planted = toGrassPlanted(items);
  const plantedModels = toGrassPlantedModels(items);

  return Fn(() => {
    const cell = int(instanceIndex).toVar();

    If(cell.greaterThanEqual(toGrassRingCells(ring)), () => {
      Return();
    });

    const slot: IGrassRingSlot = toGrassRingSlot(uniforms, ring, cell);
    const held = keys.element(uint(cell).mul(GRASS_CACHE_KEY_WORDS).add(3)).toVar();

    // A stale cell still waiting on the schedule holds another slot's tufts, or tufts of another density.
    If(held.equal(0).or(isGrassCellCurrent(keys, cell, slot, uint(uniforms.generation)).not()), () => {
      Return();
    });

    const ground = cellShapes.element(cell).toVar();

    // The slot's distance from the eye fades every tuft in it, by squared metres from one to `dm_fade`.
    const center = vec3(float(slot.x).mul(2).add(1), ground.x, float(slot.z).mul(2).add(1));
    const offset = uniforms.eye.sub(center);
    const distance = dot(offset, offset).toVar();
    const fadeLimit = uniforms.fade.mul(uniforms.fade);

    If(distance.greaterThan(fadeLimit), () => {
      Return();
    });

    // `testSAABB`: a slot outside the view passes over whole. Its sphere is grown by the largest tuft, since a tuft
    // reaches past the ground its slot's box holds.
    const bound = sqrt(float(2).add(ground.y.mul(ground.y)))
      .add(GRASS_BOX_GROWTH)
      .add(uniforms.height.mul(level.tuftReach));

    If(isGrassOutsideView(uniforms, vec3(center.x, center.y, center.z.negate()), bound), () => {
      Return();
    });

    const shrink = float(1)
      .sub(select(distance.lessThan(1), float(0), distance.sub(1).div(fadeLimit.sub(1))))
      .toVar();
    const first = uint(cell).mul(cache.perCell).toVar();

    loopNamed({ end: held, name: "tuft", start: uint(0), type: "uint" }, (tuft: Node<"uint">) => {
      const from = first.add(tuft).mul(GRASS_CACHE_VECTORS);
      const placed = cached.element(from).toVar();
      const kept = cached.element(from.add(1)).toVar();
      const model = uint(kept.y).toVar();
      const shape = shapes.element(model.mul(2)).toVar();
      const flags = shapes.element(model.mul(2).add(1)).toVar();

      // A tuft's screen area by its slot's distance, against the engine's thresholds.
      const shrunk = kept.x.mul(uniforms.height).mul(shrink).toVar();
      const area = shrunk.mul(shrunk).mul(shape.z).mul(shape.z).div(max(distance, 0.0001)).toVar();

      If(area.lessThan(discard), () => {
        Continue();
      });

      const isWaving = flags.x.greaterThan(0.5).and(area.greaterThan(discard.mul(16)));
      const place = placed.xyz;

      If(isGrassOutsideView(uniforms, place.add(vec3(0, shape.w.mul(shrunk).mul(0.5), 0)), shape.z.mul(shrunk)), () => {
        Continue();
      });

      const at = (atomicAdd(counts.element(models), uint(1)) as unknown as Node<"uint">).toVar();

      If(at.greaterThanEqual(uint(capacity)), () => {
        Continue();
      });

      planted.element(at.mul(GRASS_ITEM_VECTORS)).assign(vec4(place, placed.w));
      planted
        .element(at.mul(GRASS_ITEM_VECTORS).add(1))
        .assign(vec4(shrunk, ground.z, ground.w, select(isWaving, kept.z, 0)));
      plantedModels.element(at).assign(model);
      atomicAdd(counts.element(model), uint(1));
    });
  })().compute(cache.cells);
}
