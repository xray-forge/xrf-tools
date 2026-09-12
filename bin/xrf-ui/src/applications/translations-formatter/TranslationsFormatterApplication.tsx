import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { TranslationsFormatterService } from "@/applications/translations-formatter/services/formatter";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

import { TranslationsFormatResult } from "./components/TranslationsFormatResult";

export function TranslationsFormatterApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const formatterService: TranslationsFormatterService = useInjection(TranslationsFormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over sources it is still rewriting.
  const job: Nullable<IJobState> = formatterService.operation.job;
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

  const onCancel = useCallback(() => formatterService.operation.cancel(), [formatterService]);

  useEffect(() => {
    formatterService.operation.reset();
  }, [directory, isCheck, formatterService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!sources.isValid}
      title={isCheck ? "Check translations formatting" : "Format translation sources"}
      description={
        isCheck
          ? "Reports which sources are not normalized. Nothing is written."
          : "Rewrites every unformatted source in the directory in place."
      }
      error={formatterService.operation.error ?? undefined}
      submitLabel={isCheck ? "Check" : "Format"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        formatterService.operation.result ? (
          <TranslationsFormatResult isCheck={isCheck} result={formatterService.operation.result} />
        ) : null
      }
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
    </PickerForm>
  );
}
