import { Nullable } from "@xrf/types";
import { materialReference, nodeObject } from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { ISurfaceInputs } from "#/material/surface-inputs";
import { ESurfaceSlot, getSurfaceSlotPlaceholder } from "#/material/surface-slot";
import { SurfaceSlotTextureNode } from "#/material/surface-slot-texture-node";
import { ISurfaceValues } from "#/material/surface-values";

/** Where a material carries its numbers, read per object by name. */
const VALUES: string = "surfaceValues";

/**
 * @param name - One of a surface's numbers.
 * @param type - Its shader type.
 * @returns A uniform of it, read from each object's material as that object is refreshed.
 */
function toValue<T extends "float" | "vec3">(name: keyof ISurfaceValues, type: T): Node<T> {
  return materialReference(`${VALUES}.${name}`, type) as unknown as Node<T>;
}

/**
 * @param bias - The mip levels every sample is moved by, or null for a material drawing no screen pixels.
 * @returns What a variant's shader reads per object, every sampler and uniform one node the variant's materials share.
 */
export function toSurfaceInputs(bias: Nullable<Node<"float">>): ISurfaceInputs {
  return {
    alphaReference: toValue("alphaReference", "float"),
    color: toValue("color", "vec3"),
    detailScale: toValue("detailScale", "float"),
    sample: (slot: ESurfaceSlot, coordinates: Node<"vec2">, isBiased: boolean = true): TextureNode => {
      const sampler: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(getSurfaceSlotPlaceholder(slot), coordinates);

      sampler.slot = slot;

      const node: TextureNode = nodeObject(sampler) as unknown as TextureNode;

      return bias && isBiased ? node.bias(bias) : node;
    },
    slice: toValue("slice", "float"),
    tiling: toValue("tiling", "float"),
    unbiased: (): ISurfaceInputs => toSurfaceInputs(null),
  };
}
