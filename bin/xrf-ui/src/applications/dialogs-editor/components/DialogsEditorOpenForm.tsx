import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { createRoots } from "@/core/assets/lib/roots";
import { DialogProjectMode } from "@/core/bindings/types/xrf-dialog";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

const MODE_LABELS: Record<DialogProjectMode, string> = {
  gamedata: "Game data",
  source: "Project sources",
};

const MODE_DESCRIPTIONS: Record<DialogProjectMode, string> = {
  gamedata: "Dialogs under configs\\gameplay, their text under configs\\text, as the game ships them.",
  source: "Dialogs under configs\\gameplay, their text as multi-language JSON in translations.",
};

export function DialogsEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const dialogsService: DialogsService = useInjection(DialogsService);
  const pathsService: PathsService = useInjection(PathsService);

  const isLoading: boolean = dialogsService.project.isLoading;

  // Gamedata rather than source, matching the crate: dialog tooling aims at shipped game data first.
  const [mode, setMode] = useState<DialogProjectMode>("gamedata");

  const dialogs: IPathField = usePathField({
    application: EApplicationId.DIALOGS_EDITOR,
    id: "directory",
    title: "Select root to read dialogs from",
    isDirectory: true,
    isDisabled: isLoading,
    seed: () => resolveExistingPathRole(EPathRole.CONTENT_ROOT, pathsService.paths),
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

      <FormRow label={"Layout"} description={"Where the dialogs keep their text, and therefore what resolves"}>
        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={mode}
          disabled={isLoading}
          onChange={(_, next: Nullable<DialogProjectMode>) => next && setMode(next)}
        >
          {(Object.keys(MODE_LABELS) as Array<DialogProjectMode>).map((it: DialogProjectMode) => (
            <ToggleButton key={it} value={it}>
              {MODE_LABELS[it]}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </FormRow>
    </PickerForm>
  );
}
