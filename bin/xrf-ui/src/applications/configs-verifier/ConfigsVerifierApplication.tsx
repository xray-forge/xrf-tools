import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ConfigsVerifyResult } from "@/applications/configs-verifier/components/ConfigsVerifyResult";
import { VerifierService } from "@/applications/configs-verifier/services/verifier";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function ConfigsVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const verifierService: VerifierService = useInjection(VerifierService);

  // The run rather than this view's own flag: a verification survives the window being reloaded, so returning here
  // finds it again instead of showing an idle form over a project it is still reading.
  const job: Nullable<IJobState> = verifierService.operation.job;
  const isRunning: boolean = verifierService.operation.isRunning;

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_VERIFIER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const directory: Nullable<string> = configs.value;

  const [isDltx, setDltx] = useState<boolean>(false);

  const onVerify = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Verifying:", { directory, isDltx });

    await verifierService.verify(directory, isDltx);
  }, [directory, isDltx, log, verifierService]);

  const onCancel = useCallback(() => verifierService.operation.cancel(), [verifierService]);

  useEffect(() => {
    verifierService.operation.reset();
  }, [directory, isDltx, verifierService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!configs.isValid}
      title={"Verify LTX configs"}
      description={"Checks every LTX file in the directory. Nothing is written."}
      error={verifierService.operation.error ?? undefined}
      submitLabel={"Verify"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        verifierService.operation.result ? <ConfigsVerifyResult result={verifierService.operation.result} /> : null
      }
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Configs directory"}
        description={"Directory of LTX files to validate"}
        field={configs}
      />

      <CheckboxFormRow
        label={"DLTX"}
        description={"Read configs using DLTX patch rules"}
        isChecked={isDltx}
        isDisabled={isRunning}
        onChange={setDltx}
      />
    </PickerForm>
  );
}
