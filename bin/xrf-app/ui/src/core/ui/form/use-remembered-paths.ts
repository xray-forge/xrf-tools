import { EToolId } from "@/core/routing/tool";
import { getFieldValueStorageKey, useStoredValue } from "@/core/storage";
import { PATH_LIST_CODEC } from "@/core/ui/form/path-list-codec";

export interface IRememberedPathsOptions {
  /** The tool that owns this field. */
  tool: EToolId;
  /** Names the field inside its own form, for example `objects`. Only unique within the tool. */
  id: string;
}

/**
 * A list of paths remembered between sessions under the key a path field of the same id would use, so a hand-over
 * fills it as it fills a path field, through {@link PATH_LIST_CODEC}.
 *
 * @param options - Field identity.
 * @param options.tool - Tool the field belongs to, used to scope persistence.
 * @param options.id - Field name within that tool, used for persistence.
 * @returns The paths, none where nothing was remembered, and a setter that also records them.
 */
export function useRememberedPaths({
  tool,
  id,
}: IRememberedPathsOptions): [Array<string>, (paths: Array<string>) => void] {
  return useStoredValue(getFieldValueStorageKey(tool, id), PATH_LIST_CODEC);
}
