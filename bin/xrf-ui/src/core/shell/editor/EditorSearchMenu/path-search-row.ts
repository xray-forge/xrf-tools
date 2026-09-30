import { IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { ILogicalPathParts, splitLogicalPath } from "@/lib/path/separator";

/**
 * A search result for a logical path: its name, with the directory it sits in beneath.
 *
 * @param path - The logical path the row stands for, which is also its id.
 * @returns The row.
 */
export function toPathSearchRow(path: string): IEditorSearchResultRow {
  const { name, directory }: ILogicalPathParts = splitLogicalPath(path);

  return { id: path, label: name, description: directory ?? undefined };
}
