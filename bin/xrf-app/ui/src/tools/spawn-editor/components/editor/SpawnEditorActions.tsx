import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ImportExportIcon } from "@xrf/ui/icons/import-export";
import { SaveIcon } from "@xrf/ui/icons/save";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useState } from "react";

import { SPAWN_FILE_FILTERS } from "@/core/path/file-filters";
import { askToOpenDirectory, askToSave } from "@/core/path/native-dialog";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

/**
 * Commands that act on the open spawn file.
 */
export function SpawnEditorActions({
  "data-testid": dataTestId = "spawn-editor-actions",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spawnEditorService: SpawnEditorService = useInjection(SpawnEditorService);

  const [exportPath, setExportPath] = useState<Nullable<string>>(null);

  const isBusy: boolean = spawnEditorService.isBusy;

  const onSave = useCallback(async () => {
    // The save dialog asks about overwriting an existing file itself, so its answer is the force and there is no second
    // prompt here.
    const path: Nullable<string> = await askToSave({
      title: "Save spawn file",
      start: spawnEditorService.path,
      filters: SPAWN_FILE_FILTERS,
    });

    if (path) {
      await spawnEditorService.saveFile(path, true);
    }
  }, [spawnEditorService]);

  const onPickExportPath = useCallback(async () => {
    const path: Nullable<string> = await askToOpenDirectory({ title: "Export spawn file" });

    if (path) {
      setExportPath(path);
    }
  }, []);

  const onConfirmExport = useCallback(() => {
    const path: Nullable<string> = exportPath;

    setExportPath(null);

    // The confirmation said what is replaced, so it is the force.
    if (path) {
      void spawnEditorService.saveUnpackedDirectory(path, true);
    }
  }, [exportPath, spawnEditorService]);

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
