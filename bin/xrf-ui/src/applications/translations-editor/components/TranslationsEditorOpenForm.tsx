import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { createRoots } from "@/core/assets/lib/roots";
import { TranslationProjectMode } from "@/core/bindings/types/xrf-translation";
import { EApplicationId } from "@/core/routing/application";
import { getPathIfExists, getProjectEnginePath } from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { FormRow } from "@/core/ui/form/FormRow";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

const MODE_LABELS: Record<TranslationProjectMode, string> = {
  source: "Project sources",
  gamedata: "Game data",
};

const MODE_DESCRIPTIONS: Record<TranslationProjectMode, string> = {
  source: "Multi-language JSON and language-suffixed XML, as the project authors them.",
  gamedata: "A text directory whose subdirectories are languages, as the game ships them.",
};

export function TranslationsEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const translationsService: TranslationsService = useInjection(TranslationsService);
  const projectService: ProjectService = useInjection(ProjectService);

  const isLoading: boolean = translationsService.project.isLoading;

  const [mode, setMode] = useState<TranslationProjectMode>("source");

  const translations: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_EDITOR,
    id: "directory",
    title: "Select root to read translations from",
    isDirectory: true,
    isDisabled: isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getPathIfExists(getProjectEnginePath(projectService.xrfProjectPath)) : null,
  });

  const path: Nullable<string> = translations.value;

  const onOpen = useCallback(() => {
    if (translations.value) {
      void translationsService.openProject(createRoots([translations.value]), mode);
    } else {
      log.info("Cannot open translations without a path");
    }
  }, [log, mode, translations.value, translationsService]);

  // Probing only moves the control the user can still move back. It never decides the mode, because
  // the two layouts save to different files and a wrong guess would pick what a save overwrites.
  useEffect(() => {
    let isCurrent: boolean = true;

    if (path) {
      void translationsService.detectMode(createRoots([path])).then((detected: Nullable<TranslationProjectMode>) => {
        if (isCurrent && detected) {
          setMode(detected);
        }
      });
    }

    return () => {
      isCurrent = false;
    };
  }, [path, translationsService]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!translations.isValid}
      title={"Open translations"}
      description={MODE_DESCRIPTIONS[mode]}
      error={translationsService.project.error ? String(translationsService.project.error) : null}
      submitLabel={"Open"}
      onSubmit={onOpen}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Root directory"}
        description={"Gamedata tree, installation, or project sources holding the localization tables"}
        field={translations}
      />

      <FormRow label={"Layout"} description={"What the directory holds, and therefore what a save writes"}>
        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={mode}
          disabled={isLoading}
          onChange={(_, next: Nullable<TranslationProjectMode>) => next && setMode(next)}
        >
          {(Object.keys(MODE_LABELS) as Array<TranslationProjectMode>).map((it: TranslationProjectMode) => (
            <ToggleButton key={it} value={it}>
              {MODE_LABELS[it]}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </FormRow>
    </PickerForm>
  );
}
