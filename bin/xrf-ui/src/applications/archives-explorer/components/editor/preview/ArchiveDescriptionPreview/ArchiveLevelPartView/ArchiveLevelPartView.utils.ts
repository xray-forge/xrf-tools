import { Nullable } from "@xrf/types";

import { formatNumber } from "@/lib/format/number";

/**
 * The grid one navigation node covers.
 *
 * @param nodeSize - Spacing between nodes on the ground plane.
 * @param nodeHeight - Height one node spans.
 * @returns A phrase for the node's own size.
 */
export function formatNodeSize(nodeSize: Nullable<number>, nodeHeight: Nullable<number>): string {
  return `${formatNumber(nodeSize, 2)} m apart, ${formatNumber(nodeHeight, 2)} m tall`;
}
