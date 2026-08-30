import { Checkbox, FormControlLabel } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { ConfigsFormatResult } from "@/applications/configs-formatter/components/ConfigsFormatResult";
import { FormatterService } from "@/applications/configs-formatter/services/formatter";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function ConfigsFormatterApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const formatterService: FormatterService = useInjection(FormatterService);

  const [isCheck, setIsCheck] = useState(true);

  // The run rather than this view's own flag: it survives the window being reloaded, so returning here finds it again
  // instead of showing an idle form over files it is still rewriting.
  const job: Nullable<IJobState> = formatterService.job;
  const isRunning: boolean = Boolean(job);

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_FORMATTER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.CONFIGS, pathsService.paths),
  });

  const directory: Nullable<string> = configs.value;

  const onFormat = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Performing format command:", isCheck, directory);

    await formatterService.format(directory, isCheck);
  }, [directory, formatterService, isCheck, log]);

  const onCancel = useCallback(() => formatterService.cancel(), [formatterService]);

  const onCheckModeChange = useCallback(
    (_: ChangeEvent<HTMLInputElement>, checked: boolean) => {
      formatterService.reset();
      setIsCheck(checked);
    },
    [formatterService]
  );

  useEffect(() => {
    formatterService.reset();
  }, [directory, formatterService]);

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
      error={formatterService.error ?? undefined}
      submitLabel={isCheck ? "Check" : "Format"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        formatterService.result ? <ConfigsFormatResult isCheck={isCheck} result={formatterService.result} /> : null
      }
      onSubmit={onFormat}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Configs directory"}
        description={"Directory of LTX files to format"}
        field={configs}
      />

      <FormControlLabel
        control={<Checkbox disabled={isRunning} checked={isCheck} onChange={onCheckModeChange} />}
        label={"Check only"}
      />
    </PickerForm>
  );
}
