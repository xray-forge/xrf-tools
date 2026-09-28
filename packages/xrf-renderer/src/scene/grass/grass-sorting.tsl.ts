import { atomicAdd, atomicLoad, atomicStore, Fn, If, instanceIndex, Return, select, storage, uint } from "three/tsl";
import { ComputeNode, Node } from "three/webgpu";

import { IGrassBuffers } from "#/scene/grass/grass-buffers";
import { GRASS_ITEM_VECTORS } from "#/scene/grass/grass-item-buffers";
import {
  toGrassCounts,
  toGrassCursors,
  toGrassPlanted,
  toGrassPlantedModels,
  toGrassSorted,
} from "#/scene/grass/grass-items.tsl";
import { IGrassLevelBuffers } from "#/scene/grass/grass-level-buffers";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/**
 * @param level - The level's buffers.
 * @returns The pass zeroing the frame's counts.
 */
export function createGrassClear(level: IGrassLevelBuffers): ComputeNode {
  const counts = toGrassCounts(level);

  return Fn(() => {
    atomicStore(counts.element(instanceIndex), uint(0));
  })().compute(level.modelCount + 1);
}

/**
 * @param level - The level's buffers.
 * @returns The pass laying each model's items out after the last's and writing its draw, one thread.
 */
export function createGrassArrange(level: IGrassLevelBuffers): ComputeNode {
  const models: number = level.modelCount;
  const counts = toGrassCounts(level);
  const cursors = toGrassCursors(level);

  return Fn(() => {
    const args = storage(level.args, "uint", Math.max(models, 1) * STATIC_DRAW_ARGUMENTS);
    const starts = storage(level.starts, "uint", Math.max(models, 1));
    const first = uint(0).toVar();

    for (let model = 0; model < models; model++) {
      const count = (atomicLoad(counts.element(model)) as unknown as Node<"uint">).toVar();
      const at: number = model * STATIC_DRAW_ARGUMENTS;

      args.element(at).assign(uint(level.indexCounts[model]));
      args.element(at + 1).assign(count);
      args.element(at + 2).assign(uint(0));
      args.element(at + 3).assign(uint(0));
      // No first instance, which a device without `indirect-first-instance` would ignore: the draw reads its start.
      args.element(at + 4).assign(uint(0));
      starts.element(model).assign(first);
      atomicStore(cursors.element(model), first);
      first.addAssign(count);
    }
  })().compute(1);
}

/**
 * @param buffers - What the grass is planted into.
 * @returns The pass sorting the items into each model's range, a thread an item; its dispatch is what the frame may
 *   plant, set before each frame's.
 */
export function createGrassScatter(buffers: IGrassBuffers): ComputeNode {
  const { level, items } = buffers;
  const { capacity } = items;
  const counts = toGrassCounts(level);
  const cursors = toGrassCursors(level);
  const planted = toGrassPlanted(items).toReadOnly();
  const plantedModels = toGrassPlantedModels(items).toReadOnly();
  const sorted = toGrassSorted(items);

  return Fn(() => {
    const total = (atomicLoad(counts.element(level.modelCount)) as unknown as Node<"uint">).toVar();

    If(instanceIndex.greaterThanEqual(select(total.lessThan(uint(capacity)), total, uint(capacity))), () => {
      Return();
    });

    const at = (
      atomicAdd(cursors.element(plantedModels.element(instanceIndex)), uint(1)) as unknown as Node<"uint">
    ).toVar();

    for (let vector = 0; vector < GRASS_ITEM_VECTORS; vector++) {
      sorted
        .element(at.mul(GRASS_ITEM_VECTORS).add(vector))
        .assign(planted.element(instanceIndex.mul(GRASS_ITEM_VECTORS).add(vector)));
    }
  })().compute(capacity);
}
