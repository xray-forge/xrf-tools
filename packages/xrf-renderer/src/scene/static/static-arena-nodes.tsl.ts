import { storage } from "three/tsl";
import { StorageBufferAttribute, StorageBufferNode } from "three/webgpu";

/**
 * @param attribute - An arena's words or indices.
 * @returns The node every shader over the arena reads them through, pointed at a grown buffer by its value.
 */
export function createArenaNode(attribute: StorageBufferAttribute): StorageBufferNode<"uint"> {
  return storage(attribute, "uint", attribute.count).toReadOnly();
}
