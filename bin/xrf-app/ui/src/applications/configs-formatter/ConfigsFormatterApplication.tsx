import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { FormatCommandResult, IFormatCommandNouns } from "@xrf/ui/command-result/FormatCommandResult";
import { CheckboxFormRow } from "@xrf/ui/form";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ConfigsFormatterService } from "@/applications/configs-formatter/services/configs-formatter.service";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

/** What this formatter's items are called in its result. */
const CONFIGS_FORMAT_NOUNS: IFormatCommandNouns = { column: "File", everything: "files", item: "file" };

export function ConfigsFormatterApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const formatterService: ConfigsFormatterService = useInjection(ConfigsFormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over files it is still rewriting.
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

  useEffect(() => {
    formatterService.operation.reset();
  }, [directory, isCheck, formatterService]);

  return (
    <JobPickerForm
      operation={formatterService.operation}
      isSubmitDisabled={!configs.isValid}
      title={isCheck ? "Check LTX formatting" : "Format LTX configs"}
      description={
        isCheck
          ? "Reports which files are badly formatted. Nothing is written."
          : "Rewrites every badly formatted file in the directory in place."
      }
      submitLabel={isCheck ? "Check" : "Format"}
      renderResult={(result) => (
        <FormatCommandResult
          data-testid={"configs-format-result"}
          isCheck={isCheck}
          nouns={CONFIGS_FORMAT_NOUNS}
          result={result}
        />
      )}
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
    </JobPickerForm>
  );
}
