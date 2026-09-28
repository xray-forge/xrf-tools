import { Fn, uint, varying } from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { IClusterSource, toClusterSource } from "#/geometry/cluster-source";
import { toClusterEntry } from "#/shader/cluster-vertex.tsl";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { SURFACE_NO_ROW } from "#/uniforms/surface-table";

/**
 * @param buffers - What static draws are placed by.
 * @returns The surface table's row of the slot whose cluster the instance draws, read once a vertex and carried flat:
 *   the cluster names its slot, and the slot's record its row.
 */
export function toSurfaceRow(buffers: StaticDrawBuffers): Node<"uint"> {
  return varying(
    Fn((_: [], builder: NodeBuilder): Node<"uint"> => {
      const source: IClusterSource | null = toClusterSource(builder.geometry);

      // Only a static batch draws by the table; a material staged over another layout compiles, and never draws so.
      if (!source) {
        return uint(SURFACE_NO_ROW);
      }

      const slot: Node<"uint"> = (source.rangeNode.element(toClusterEntry(builder).x) as unknown as Node<"uvec4">).w;

      return (buffers.slotWords.element(slot.mul(2).add(1)) as unknown as Node<"uvec4">).z;
    })()
  ) as unknown as Node<"uint">;
}
