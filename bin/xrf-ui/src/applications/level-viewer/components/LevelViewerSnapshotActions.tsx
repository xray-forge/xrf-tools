import { join } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";

import { LevelViewerImportDialog } from "@/applications/level-viewer/components/LevelViewerImportDialog";
import { ILevelViewerSnapshot, ILevelViewerSnapshotEntry } from "@/applications/level-viewer/lib/level-viewer-snapshot";
import {
  getLevelViewerSnapshotFolder,
  LEVEL_VIEWER_SNAPSHOT_FILTERS,
  readLevelViewerSnapshot,
  toLevelViewerSnapshotName,
  writeLevelViewerSnapshot,
} from "@/applications/level-viewer/lib/level-viewer-snapshot-files";
import { LEVEL_VIEWER_FILE_MENU_EXTENSION } from "@/applications/level-viewer/menus";
import { ILevelViewerSnapshotApplied, LevelViewerSnapshotService } from "@/applications/level-viewer/services/snapshot";
import { useEditorMenus } from "@/core/shell/editor-shell";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { Logger, useLogger } from "@/lib/logging";

/** A snapshot read for import, with its entries set against the view as it was read. */
interface ILevelViewerImport {
  fileName: string;
  snapshot: ILevelViewerSnapshot;
  entries: Array<ILevelViewerSnapshotEntry>;
}

/**
 * The level viewer's File › Export view… and Import view…: the view written to a file the owner picks, in the
 * application's own folder of views by default, and a file read back through the import dialog.
 */
export function LevelViewerSnapshotActions(): ReactElement {
  const snapshotService: LevelViewerSnapshotService = useInjection(LevelViewerSnapshotService);
  const log: Logger = useLogger(__MODULE_NAME__);
  const [importing, setImporting] = useState<Nullable<ILevelViewerImport>>(null);
  const [result, setResult] = useState<Nullable<ILevelViewerSnapshotApplied>>(null);
  const [failure, setFailure] = useState<Nullable<string>>(null);

  const fail = useCallback(
    (what: string, error: unknown): void => {
      log.error(`Failed to ${what}:`, error);
      setFailure(error instanceof Error ? error.message : String(error));
    },
    [log]
  );

  const onExport = useCallback(async (): Promise<void> => {
    try {
      const snapshot: ILevelViewerSnapshot = await snapshotService.exportSnapshot();
      const path: Nullable<string> = await save({
        defaultPath: await join(
          await getLevelViewerSnapshotFolder(),
          toLevelViewerSnapshotName(snapshotService.level, new Date())
        ),
        filters: LEVEL_VIEWER_SNAPSHOT_FILTERS,
        title: "Export view",
      });

      if (path) {
        log.info("Exported the view:", await writeLevelViewerSnapshot(path, snapshot));
      }
    } catch (error: unknown) {
      fail("export the view", error);
    }
  }, [fail, log, snapshotService]);

  const onImport = useCallback(async (): Promise<void> => {
    try {
      const path: Nullable<string> = (await open({
        defaultPath: await getLevelViewerSnapshotFolder(),
        directory: false,
        filters: LEVEL_VIEWER_SNAPSHOT_FILTERS,
        multiple: false,
        title: "Import view",
      })) as Nullable<string>;

      if (path) {
        const snapshot: ILevelViewerSnapshot = await readLevelViewerSnapshot(path);

        setResult(null);
        setImporting({ entries: snapshotService.compare(snapshot), fileName: getPathName(path), snapshot });
      }
    } catch (error: unknown) {
      fail("read the view", error);
    }
  }, [fail, snapshotService]);

  const onApply = useCallback(
    async (entries: Array<ILevelViewerSnapshotEntry>): Promise<void> => {
      try {
        setResult(await snapshotService.apply(entries));
      } catch (error: unknown) {
        setImporting(null);
        fail("import the view", error);
      }
    },
    [fail, snapshotService]
  );

  const onCloseImport = useCallback(() => {
    setImporting(null);
    setResult(null);
  }, []);

  const onCloseFailure = useCallback(() => setFailure(null), []);

  useEditorMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_FILE_MENU_EXTENSION.sections.view,
        entries: [
          { kind: "action", id: "level-viewer/export-view", label: "Export view…", onSelect: () => void onExport() },
          { kind: "action", id: "level-viewer/import-view", label: "Import view…", onSelect: () => void onImport() },
        ],
      },
    ],
    [onExport, onImport]
  );

  return (
    <>
      {importing ? (
        <LevelViewerImportDialog
          fileName={importing.fileName}
          snapshot={importing.snapshot}
          entries={importing.entries}
          level={snapshotService.level}
          result={result}
          onApply={(entries: Array<ILevelViewerSnapshotEntry>) => void onApply(entries)}
          onClose={onCloseImport}
        />
      ) : null}

      <ConfirmDialog
        isOpen={failure !== null}
        title={"Level view"}
        description={failure}
        confirmLabel={"OK"}
        onConfirm={onCloseFailure}
        onClose={onCloseFailure}
      />
    </>
  );
}
