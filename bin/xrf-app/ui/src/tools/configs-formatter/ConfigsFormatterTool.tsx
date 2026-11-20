import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { FormatCommandResult, IFormatCommandNouns } from "@xrf/ui/command-result";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { ConfigsFormatterService } from "@/tools/configs-formatter/services/configs-formatter.service";

interface IFormatRequest {
  directory: string;
  /** Whether the run only reports, writing nothing. */
  isCheck: boolean;
}

/** What this formatter's items are called in its result. */
const CONFIGS_FORMAT_NOUNS: IFormatCommandNouns = { column: "File", everything: "files", item: "file" };

export function ConfigsFormatterTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const formatterService: ConfigsFormatterService = useInjection(ConfigsFormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over files it is still rewriting.
  const isRunning: boolean = formatterService.operation.isRunning;

  const configs: IPathField = usePathField({
    tool: EToolId.CONFIGS_FORMATTER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const request: Nullable<IFormatRequest> = useMemo(
    () => (configs.value ? { directory: configs.value, isCheck } : null),
    [configs.value, isCheck]
  );

  const onFormat = useCallback(
    async (submitted: IFormatRequest) => {
      log.info("Performing format command:", submitted.isCheck, submitted.directory);

      await formatterService.format(submitted.directory, submitted.isCheck);
    },
    [formatterService, log]
  );

  return (
    <JobPickerForm
      operation={formatterService.operation}
      request={request}
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
