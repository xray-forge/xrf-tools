import { default as ImportExportIcon } from "@mui/icons-material/ImportExport";
import { default as SaveIcon } from "@mui/icons-material/Save";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { SpawnFileService } from "@/core/spawn/services";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { Nullable } from "@/lib/types/general";

/**
 * Commands that act on the open spawn file.
 */
export function SpawnEditorActions(): ReactElement {
  const spawnFileService: SpawnFileService = useInjection(SpawnFileService);

  const [exportPath, setExportPath] = useState<Nullable<string>>(null);

  const isBusy: boolean = spawnFileService.isBusy;

  const onSave = useCallback(async () => {
    // The save dialog asks about overwriting an existing file itself, so there is no second prompt here.
    const path: Nullable<string> = await dialog.save({
      title: "Save spawn file",
      filters: [{ name: "spawn", extensions: ["spawn"] }],
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
    <>
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
    </>
  );
}
