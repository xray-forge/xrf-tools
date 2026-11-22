import { Nullable } from "@xrf/types";

import { EToolId } from "@/core/routing/tool";
import { getFieldValueStorageKey, setLocalStorageValueSafe } from "@/core/storage";
import { PATH_LIST_CODEC } from "@/core/ui/form/path-list-codec";

/** What a hand-over fills one field with: a path field's path, null to empty it, or a list field's paths. */
export type THandedOverPath = Nullable<string> | ReadonlyArray<string>;

/**
 * Fills another tool's path fields before it is opened, as picking each path there would, since each `usePathField`
 * and `useRememberedPaths` reads its remembered value as it mounts. A field handed null or no paths opens empty; one
 * left out keeps what it remembered.
 *
 * @param tool - The tool the fields belong to.
 * @param fields - Each field's path or paths, by the field's id within that tool.
 */
export function handOverPathFields(tool: EToolId, fields: Readonly<Record<string, THandedOverPath>>): void {
  for (const [id, path] of Object.entries(fields)) {
    setLocalStorageValueSafe(
      getFieldValueStorageKey(tool, id),
      typeof path === "string" || path === null ? path : PATH_LIST_CODEC.write([...path])
    );
  }
}
