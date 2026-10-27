import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { LevelViewerImportDialog } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog";
import {
  ILevelViewerImport,
  ILevelViewerSnapshotEntry,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/applications/level-viewer/menus";
import { LevelViewerSnapshotService } from "@/applications/level-viewer/services/level-viewer-snapshot";
import { useEditorMenus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";

/**
 * The level viewer's File › Export view… and Import view…, offered while a level is open, and the import dialog; what
 * fails with either is noticed.
 */
export function LevelViewerSnapshotActions(): ReactElement {
  const snapshotService: LevelViewerSnapshotService = useInjection(LevelViewerSnapshotService);
  const importing: Nullable<ILevelViewerImport> = snapshotService.importing;
  const isDisabled: boolean = snapshotService.level === null;

  const onApply = useCallback(
    (entries: Array<ILevelViewerSnapshotEntry>) => void snapshotService.applyImport(entries),
    [snapshotService]
  );

  useEditorMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_FILE_MENU_EXTENSION.sections.view,
        entries: [
          {
            kind: "action",
            id: "level-viewer/export-view",
            label: "Export view…",
            isDisabled,
            onSelect: () => void snapshotService.exportView(),
          },
          {
            kind: "action",
            id: "level-viewer/import-view",
            label: "Import view…",
            isDisabled,
            onSelect: () => void snapshotService.beginImport(),
          },
        ],
      },
    ],
    [isDisabled, snapshotService]
  );

  return importing ? (
    <LevelViewerImportDialog
      fileName={importing.fileName}
      snapshot={importing.snapshot}
      entries={importing.entries}
      level={snapshotService.level}
      result={snapshotService.applied.value}
      isApplying={snapshotService.applied.isLoading}
      onApply={onApply}
      onClose={snapshotService.closeImport}
    />
  ) : (
    <></>
  );
}
