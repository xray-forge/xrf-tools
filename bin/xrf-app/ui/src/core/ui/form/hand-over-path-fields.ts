import { Nullable } from "@xrf/types";

import { EToolId } from "@/core/routing/tool";
import { getFieldValueStorageKey, setLocalStorageValueSafe } from "@/core/storage";

/**
 * Fills another tool's path fields before it is opened, as picking each path there would, since each `usePathField`
 * reads its remembered value as it mounts. A field handed null opens empty; one left out keeps what it remembered.
 *
 * @param tool - The tool the fields belong to.
 * @param fields - Each field's path, or null to empty it, by the field's id within that tool.
 */
export function handOverPathFields(tool: EToolId, fields: Readonly<Record<string, Nullable<string>>>): void {
  for (const [id, path] of Object.entries(fields)) {
    setLocalStorageValueSafe(getFieldValueStorageKey(tool, id), path);
  }
}
