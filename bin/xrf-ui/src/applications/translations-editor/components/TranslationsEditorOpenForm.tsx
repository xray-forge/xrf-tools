import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { createRoots } from "@/core/assets/lib/roots";
import { TranslationProjectMode } from "@/core/ipc/types/xrf-translation";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IChoiceFormRowOption, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

const MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TranslationProjectMode>> = [
  { value: "source", label: "Project sources" },
  { value: "gamedata", label: "Game data" },
];

const MODE_DESCRIPTIONS: Record<TranslationProjectMode, string> = {
  source: "Multi-language JSON and language-suffixed XML, as the project authors them.",
  gamedata: "A text directory whose subdirectories are languages, as the game ships them.",
};

export function TranslationsEditorOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const translationsService: TranslationsService = useInjection(TranslationsService);

  const isLoading: boolean = translationsService.project.isLoading;

  const [mode, setMode] = useState<TranslationProjectMode>("source");

  const translations: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_EDITOR,
    id: "directory",
    title: "Select root to read translations from",
    isDirectory: true,
    isDisabled: isLoading,
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

      <ChoiceFormRow
        label={"Layout"}
        description={"What the directory holds, and therefore what a save writes"}
        options={MODE_OPTIONS}
        value={mode}
        isDisabled={isLoading}
        onChange={setMode}
      />
    </PickerForm>
  );
}
