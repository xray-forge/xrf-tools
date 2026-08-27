import {
  DialogFileDescriptor,
  DialogProjectDescriptor,
  DialogSummaryDescriptor,
} from "@/core/bindings/types/xrf-dialog";
import { LOGICAL_PATH_SEPARATOR } from "@/core/ui/tree/path-tree";
import { Nullable } from "@/lib/types/general";

/** Which dialog a tree leaf stands for: the pair that addresses one. */
export interface IDialogTreeLeaf {
  logicalPath: string;
  id: string;
}

/** One dialog, as a path the shared tree builder can attach. */
export interface IDialogTreeEntry {
  path: string;
  payload: IDialogTreeLeaf;
}

/**
 * Every dialog in a project as a `<file>\<dialog id>` path.
 *
 * @param project - The open project, or nothing.
 * @returns One entry per dialog, in the order the project listed its files.
 */
export function toDialogTreeEntries(project: Nullable<DialogProjectDescriptor>): Array<IDialogTreeEntry> {
  if (!project) {
    return [];
  }

  const prefix: string = project.dialogsPrefix ? `${project.dialogsPrefix}${LOGICAL_PATH_SEPARATOR}` : "";

  return Object.entries(project.files).flatMap(([logicalPath, file]: [string, DialogFileDescriptor]) =>
    file.dialogs.map((dialog: DialogSummaryDescriptor) => ({
      path: `${toRelativePath(logicalPath, prefix)}${LOGICAL_PATH_SEPARATOR}${dialog.id}`,
      payload: { id: dialog.id, logicalPath },
    }))
  );
}

/** A file's path with the project's dialogs prefix removed, matched without case as the VFS does. */
function toRelativePath(logicalPath: string, prefix: string): string {
  return prefix && logicalPath.toLowerCase().startsWith(prefix.toLowerCase())
    ? logicalPath.slice(prefix.length)
    : logicalPath;
}
