import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { MODE_DESCRIPTIONS, MODE_OPTIONS } from "@/applications/dialogs-editor/components/DialogsEditorOpenForm.utils";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { createRoots } from "@/core/assets/lib/roots";
import { DialogProjectMode } from "@/core/bindings/types/xrf-dialog";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function DialogsEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const dialogsService: DialogsService = useInjection(DialogsService);

  const isLoading: boolean = dialogsService.project.isLoading;

  // Gamedata rather than source, matching the crate: dialog tooling aims at shipped game data first.
  const [mode, setMode] = useState<DialogProjectMode>("gamedata");

  const dialogs: IPathField = usePathField({
    application: EApplicationId.DIALOGS_EDITOR,
    id: "directory",
    title: "Select root to read dialogs from",
    isDirectory: true,
    isDisabled: isLoading,
  });

  const path: Nullable<string> = dialogs.value;

  const onOpen = useCallback(() => {
    if (dialogs.value) {
      void dialogsService.openProject(createRoots([dialogs.value]), mode);
    } else {
      log.info("Cannot open dialogs without a path");
    }
  }, [dialogs.value, dialogsService, log, mode]);

  // Probing only moves the control the user can still move back. It never decides the mode, because
  // the two layouts read text from different places and a wrong guess would show every phrase as
  // untranslated.
  useEffect(() => {
    let isCurrent: boolean = true;

    if (path) {
      void dialogsService.detectMode(createRoots([path])).then((detected: Nullable<DialogProjectMode>) => {
        if (isCurrent && detected) {
          setMode(detected);
        }
      });
    }

    return () => {
      isCurrent = false;
    };
  }, [dialogsService, path]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!dialogs.isValid}
      title={"Open dialogs"}
      description={MODE_DESCRIPTIONS[mode]}
      error={dialogsService.project.error ? String(dialogsService.project.error) : null}
      submitLabel={"Open"}
      onSubmit={onOpen}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Root directory"}
        description={"Gamedata tree, installation, or project sources holding the dialogs"}
        field={dialogs}
      />

      <ChoiceFormRow
        label={"Layout"}
        description={"Where the dialogs keep their text, and therefore what resolves"}
        options={MODE_OPTIONS}
        value={mode}
        isDisabled={isLoading}
        onChange={setMode}
      />
    </PickerForm>
  );
}
