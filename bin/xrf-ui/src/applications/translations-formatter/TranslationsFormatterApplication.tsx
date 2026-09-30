import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsFormatterService } from "@/applications/translations-formatter/services/formatter";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

import { TranslationsFormatResult } from "./components/TranslationsFormatResult";

export function TranslationsFormatterApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const formatterService: TranslationsFormatterService = useInjection(TranslationsFormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over sources it is still rewriting.
  const isRunning: boolean = formatterService.operation.isRunning;

  const sources: IPathField = usePathField({
    application: EApplicationId.TRANSLATIONS_FORMATTER,
    id: "sources",
    title: "Select translations sources",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const directory: Nullable<string> = sources.value;

  const onFormat = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Performing translations format command:", isCheck, directory);

    await formatterService.format(directory, isCheck);
  }, [directory, formatterService, isCheck, log]);

  useEffect(() => {
    formatterService.operation.reset();
  }, [directory, isCheck, formatterService]);

  return (
    <JobPickerForm
      operation={formatterService.operation}
      isSubmitDisabled={!sources.isValid}
      title={isCheck ? "Check translations formatting" : "Format translation sources"}
      description={
        isCheck
          ? "Reports which sources are not normalized. Nothing is written."
          : "Rewrites every unformatted source in the directory in place."
      }
      submitLabel={isCheck ? "Check" : "Format"}
      renderResult={(result) => <TranslationsFormatResult isCheck={isCheck} result={result} />}
      onSubmit={onFormat}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Translations directory"}
        description={"Directory of JSON translation sources to format"}
        field={sources}
      />

      <CheckboxFormRow
        label={"Check only"}
        description={"Report formatting differences without rewriting sources"}
        isChecked={isCheck}
        isDisabled={isRunning}
        onChange={setIsCheck}
      />
    </JobPickerForm>
  );
}
