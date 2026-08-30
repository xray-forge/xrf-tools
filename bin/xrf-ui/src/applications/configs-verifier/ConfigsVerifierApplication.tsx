import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect } from "react";

import { ConfigsVerifyResult } from "@/applications/configs-verifier/components/ConfigsVerifyResult";
import { VerifierService } from "@/applications/configs-verifier/services/verifier";
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

export function ConfigsVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const verifierService: VerifierService = useInjection(VerifierService);

  // The run rather than this view's own flag: a verification survives the window being reloaded, so returning here
  // finds it again instead of showing an idle form over a project it is still reading.
  const job: Nullable<IJobState> = verifierService.job;
  const isRunning: boolean = Boolean(job);

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_VERIFIER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.CONFIGS, pathsService.paths),
  });

  const directory: Nullable<string> = configs.value;

  const onVerify = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Verifying:", directory);

    await verifierService.verify(directory);
  }, [directory, log, verifierService]);

  const onCancel = useCallback(() => verifierService.cancel(), [verifierService]);

  // A different directory invalidates whatever the previous run reported.
  useEffect(() => {
    verifierService.reset();
  }, [directory, verifierService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!configs.isValid}
      title={"Verify LTX configs"}
      description={"Checks every LTX file in the directory. Nothing is written."}
      error={verifierService.error ?? undefined}
      submitLabel={"Verify"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={verifierService.result ? <ConfigsVerifyResult result={verifierService.result} /> : null}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Configs directory"}
        description={"Directory of LTX files to validate"}
        field={configs}
      />
    </PickerForm>
  );
}
