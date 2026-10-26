import { default as SaveIcon } from "@mui/icons-material/Save";
import { default as UndoIcon } from "@mui/icons-material/Undo";
import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";

export function TranslationsEditorActions(): ReactElement {
  const translationsService: TranslationsService = useInjection(TranslationsService);

  const [isDiscarding, setDiscarding] = useState<boolean>(false);

  const dirtyFiles: Array<string> = translationsService.dirtyFiles;
  const isBusy: boolean = translationsService.savingFile !== null;

  const onSaveAll = useCallback(async () => {
    await flowResult(translationsService.saveAll());
  }, [translationsService]);

  const onDiscardAll = useCallback(() => {
    translationsService.dirtyFiles.forEach((file: string) => translationsService.discardFile(file));
    setDiscarding(false);
  }, [translationsService]);

  return (
    <>
      <EditorIconAction
        label={"Save translations"}
        description={dirtyFiles.length ? `Write ${dirtyFiles.length} changed file(s)` : "Nothing to save"}
        icon={<SaveIcon />}
        isDisabled={isBusy || !dirtyFiles.length}
        onClick={() => void onSaveAll()}
      />

      <EditorIconAction
        label={"Discard translation edits"}
        description={"Throw away every unsaved edit"}
        icon={<UndoIcon />}
        isDisabled={isBusy || !dirtyFiles.length}
        onClick={() => setDiscarding(true)}
      />

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
