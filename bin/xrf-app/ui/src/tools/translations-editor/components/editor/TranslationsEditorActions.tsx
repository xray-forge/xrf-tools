import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { SaveIcon } from "@xrf/ui/icons/save";
import { UndoIcon } from "@xrf/ui/icons/undo";
import { ReactElement, useCallback, useState } from "react";

import { TranslationsService } from "@/tools/translations-editor/services/translations.service";

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
