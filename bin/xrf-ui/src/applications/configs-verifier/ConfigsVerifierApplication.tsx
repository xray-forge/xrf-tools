import { Alert } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ConfigsVerifyResult } from "@/applications/configs-verifier/components/ConfigsVerifyResult";
import { createRoots } from "@/core/assets/lib";
import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { getProjectConfigsPath } from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function ConfigsVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const projectService: ProjectService = useInjection(ProjectService);

  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Nullable<string>>(null);
  const [result, setResult] = useState<Nullable<LtxProjectVerifyResult>>(null);

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_VERIFIER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: async () => (projectService.xrfProjectPath ? getProjectConfigsPath(projectService.xrfProjectPath) : null),
  });

  const onVerify = useCallback(async () => {
    if (!configs.value) {
      return;
    }

    try {
      setIsLoading(true);
      setResult(null);
      setError(null);

      log.info("Verifying:", configs.value);

      const verified: LtxProjectVerifyResult = await configsCommands.verifyDirectory(
        createRoots([configs.value]),
        null
      );

      setResult(verified);

      notify({
        details: String(configs.value),
        severity: verified.errors.length ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
        source: EApplicationId.CONFIGS_VERIFIER,
        title: verified.errors.length
          ? `Configs did not pass validation: ${verified.errors.length} problem(s)`
          : "Configs passed validation",
      });
    } catch (caught: unknown) {
      log.error("Verify error:", caught);

      setError(String(caught));

      notify({
        details: `${configs.value}\n${String(caught)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.CONFIGS_VERIFIER,
        title: "Could not verify configs",
      });
    } finally {
      setIsLoading(false);
    }
  }, [configs.value, log, notify]);

  // A different directory invalidates whatever the previous run reported.
  useEffect(() => {
    setResult(null);
    setError(null);
  }, [configs.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!configs.isValid}
      title={"Verify LTX configs"}
      description={"Checks every LTX file in the directory. Nothing is written."}
      error={error ?? undefined}
      submitLabel={"Verify"}
      status={
        result ? (
          result.errors.length ? (
            <Alert severity={"error"}>Configs did not pass validation.</Alert>
          ) : (
            <Alert severity={"success"}>Configs passed validation.</Alert>
          )
        ) : null
      }
      result={result ? <ConfigsVerifyResult result={result} /> : null}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Configs directory"}
        description={"Directory of LTX files to validate"}
        field={configs}
      />
    </PickerForm>
  );
}
