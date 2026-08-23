import { Alert, Checkbox, FormControlLabel } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { ConfigsFormatResult } from "@/applications/configs-formatter/components/ConfigsFormatResult";
import { createWorldSpec } from "@/core/assets/lib";
import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { getProjectConfigsPath } from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function ConfigsFormatterApplication(): ReactElement {
  const log: Logger = useLogger("configs-formatter");
  const notify: TEmitNotification = useEmitNotification();

  const projectService: ProjectService = useInjection(ProjectService);

  const [isCheck, setIsCheck] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [result, setResult] = useState<Nullable<LtxProjectFormatResult>>(null);

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_FORMATTER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: async () => (projectService.xrfProjectPath ? getProjectConfigsPath(projectService.xrfProjectPath) : null),
  });

  const onFormat = useCallback(async () => {
    if (!configs.value) {
      return;
    }

    try {
      setIsLoading(true);
      setResult(null);
      setError(null);

      log.info("Performing format command:", isCheck, configs.value);

      const formatted: LtxProjectFormatResult = await (isCheck
        ? configsCommands.checkDirectoryFormat(createWorldSpec([configs.value]), null)
        : configsCommands.formatDirectory(createWorldSpec([configs.value]), null));

      setResult(formatted);

      notify({
        details: String(configs.value),
        severity: formatted.toFormat.length
          ? isCheck
            ? ENotificationSeverity.ERROR
            : ENotificationSeverity.WARNING
          : ENotificationSeverity.SUCCESS,
        source: EApplicationId.CONFIGS_FORMATTER,
        title: formatted.toFormat.length
          ? isCheck
            ? `${formatted.toFormat.length} file(s) have invalid formatting`
            : `Formatted ${formatted.toFormat.length} file(s)`
          : "All files are in correct format",
      });
    } catch (caught) {
      log.error("Format error:", caught);
      setError(String(caught));

      notify({
        details: `${configs.value}\n${String(caught)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.CONFIGS_FORMATTER,
        title: isCheck ? "Could not check formatting" : "Could not format configs",
      });
    } finally {
      setIsLoading(false);
    }
  }, [configs.value, isCheck, log, notify]);

  const onCheckModeChange = useCallback((_: ChangeEvent<HTMLInputElement>, checked: boolean) => {
    setResult(null);
    setError(null);
    setIsCheck(checked);
  }, []);

  useEffect(() => {
    setResult(null);
    setError(null);
  }, [configs.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!configs.isValid}
      title={isCheck ? "Check LTX formatting" : "Format LTX configs"}
      description={
        isCheck
          ? "Reports which files are badly formatted. Nothing is written."
          : "Rewrites every badly formatted file in the directory in place."
      }
      error={error ?? undefined}
      submitLabel={isCheck ? "Check" : "Format"}
      status={
        result ? (
          result.toFormat.length ? (
            isCheck ? (
              <Alert severity={"error"}>There are files with invalid formatting.</Alert>
            ) : (
              <Alert severity={"warning"}>Formatted {result.toFormat.length} file(s).</Alert>
            )
          ) : (
            <Alert severity={"success"}>All files are in correct format.</Alert>
          )
        ) : null
      }
      result={result ? <ConfigsFormatResult isCheck={isCheck} result={result} /> : null}
      onSubmit={onFormat}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Configs directory"}
        description={"Directory of LTX files to format"}
        field={configs}
      />

      <FormControlLabel
        control={<Checkbox disabled={isLoading} checked={isCheck} onChange={onCheckModeChange} />}
        label={"Check mode (readonly)"}
      />
    </PickerForm>
  );
}
