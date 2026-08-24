import { default as SaveIcon } from "@mui/icons-material/Save";
import { default as UndoIcon } from "@mui/icons-material/Undo";
import { IconButton, Tooltip } from "@mui/material";
import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";

export function TranslationsEditorActions(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  const [isDiscarding, setDiscarding] = useState<boolean>(false);

  const dirtyFiles: Array<string> = translationsService.dirtyFiles;
  const isBusy: boolean = translationsService.savingFile !== null;

  const onSaveAll = useCallback(async () => {
    // Sequential rather than concurrent: each save re-reads the project, so overlapping them would
    // race the descriptor every one of them returns.
    for (const file of translationsService.dirtyFiles) {
      if (!(await flowResult(translationsService.saveFile(file)))) {
        return;
      }
    }
  }, [translationsService]);

  const onDiscardAll = useCallback(() => {
    translationsService.dirtyFiles.forEach((file: string) => translationsService.discardFile(file));
    setDiscarding(false);
  }, [translationsService]);

  return (
    <>
      <Tooltip
        describeChild
        title={dirtyFiles.length ? `Write ${dirtyFiles.length} changed file(s)` : "Nothing to save"}
      >
        <span>
          <IconButton
            aria-label={"Save translations"}
            disabled={isBusy || !dirtyFiles.length}
            onClick={() => void onSaveAll()}
          >
            <SaveIcon />
          </IconButton>
        </span>
      </Tooltip>

      <Tooltip describeChild title={"Throw away every unsaved edit"}>
        <span>
          <IconButton
            aria-label={"Discard translation edits"}
            disabled={isBusy || !dirtyFiles.length}
            onClick={() => setDiscarding(true)}
          >
            <UndoIcon />
          </IconButton>
        </span>
      </Tooltip>

      <ConfirmDialog
        isDestructive={true}
        isOpen={isDiscarding}
        title={"Discard edits?"}
        description={`${dirtyFiles.length} file(s) hold edits that were never written. Discarding cannot be undone.`}
        confirmLabel={"Discard"}
        onConfirm={onDiscardAll}
        onClose={() => setDiscarding(false)}
      />
    </>
  );
}
