import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { FormatterService } from "@/applications/configs-formatter/services/formatter";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

import { ConfigsFormatResult } from "./components/ConfigsFormatResult";

export function ConfigsFormatterApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const formatterService: FormatterService = useInjection(FormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over files it is still rewriting.
  const job: Nullable<IJobState> = formatterService.operation.job;
  const isRunning: boolean = formatterService.operation.isRunning;

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_FORMATTER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const directory: Nullable<string> = configs.value;

  const onFormat = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Performing format command:", isCheck, directory);

    await formatterService.format(directory, isCheck);
  }, [directory, formatterService, isCheck, log]);

  const onCancel = useCallback(() => formatterService.operation.cancel(), [formatterService]);

  useEffect(() => {
    formatterService.operation.reset();
  }, [directory, isCheck, formatterService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!configs.isValid}
      title={isCheck ? "Check LTX formatting" : "Format LTX configs"}
      description={
        isCheck
          ? "Reports which files are badly formatted. Nothing is written."
          : "Rewrites every badly formatted file in the directory in place."
      }
      error={formatterService.operation.error ?? undefined}
      submitLabel={isCheck ? "Check" : "Format"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        formatterService.operation.result ? (
          <ConfigsFormatResult isCheck={isCheck} result={formatterService.operation.result} />
        ) : null
      }
      onSubmit={onFormat}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Configs directory"}
        description={"Directory of LTX files to format"}
        field={configs}
      />

      <CheckboxFormRow
        label={"Check only"}
        description={"Report formatting differences without rewriting files"}
        isChecked={isCheck}
        isDisabled={isRunning}
        onChange={setIsCheck}
      />
    </PickerForm>
  );
}
