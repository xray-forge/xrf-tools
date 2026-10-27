import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ImportExportIcon } from "@xrf/ui/icons/import-export";
import { SaveIcon } from "@xrf/ui/icons/save";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useState } from "react";

import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file.service";
import { SPAWN_FILE_FILTERS } from "@/core/path/file-filters";

/**
 * Commands that act on the open spawn file.
 */
export function SpawnEditorActions({
  "data-testid": dataTestId = "spawn-editor-actions",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const [exportPath, setExportPath] = useState<Nullable<string>>(null);

  const isBusy: boolean = spawnFileService.isBusy;

  const onSave = useCallback(async () => {
    // The save dialog asks about overwriting an existing file itself, so there is no second prompt here.
    const path: Nullable<string> = await dialog.save({
      title: "Save spawn file",
      filters: SPAWN_FILE_FILTERS,
    });

    if (path) {
      await spawnFileService.saveFile(path);
    }
  }, [spawnFileService]);

  const onPickExportPath = useCallback(async () => {
    const path: Nullable<string> = (await dialog.open({
      title: "Export spawn file",
      directory: true,
    })) as Nullable<string>;

    if (path) {
      setExportPath(path);
    }
  }, []);

  const onConfirmExport = useCallback(() => {
    const path: Nullable<string> = exportPath;

    setExportPath(null);

    if (path) {
      void spawnFileService.saveUnpackedDirectory(path);
    }
  }, [exportPath, spawnFileService]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex items-center", className)}>
      <EditorIconAction
        label={"Save spawn file"}
        description={"Write the open spawn file to a chosen path"}
        icon={<SaveIcon />}
        isDisabled={isBusy}
        onClick={onSave}
      />

      <EditorIconAction
        label={"Export spawn file"}
        description={"Export the spawn file into a directory"}
        icon={<ImportExportIcon />}
        isDisabled={isBusy}
        onClick={onPickExportPath}
      />

      <ConfirmDialog
        isDestructive
        confirmLabel={"Export"}
        description={`Writes one file per chunk into ${exportPath}, replacing any unpacked spawn already there.`}
        isOpen={exportPath !== null}
        title={"Export spawn file?"}
        onClose={() => setExportPath(null)}
        onConfirm={onConfirmExport}
      />
    </div>
  );
}
