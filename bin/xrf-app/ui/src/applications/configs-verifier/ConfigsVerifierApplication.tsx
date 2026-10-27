import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ConfigsVerifierService } from "@/applications/configs-verifier/services/configs-verifier.service";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { ConfigsDialectFormRow } from "@/core/ltx/components/ConfigsDialectFormRow";
import { EApplicationId } from "@/core/routing/application";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

import { ConfigsVerifyResult } from "./components/ConfigsVerifyResult";

export function ConfigsVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const verifierService: ConfigsVerifierService = useInjection(ConfigsVerifierService);

  // The run rather than this view's own flag: a verification survives the window being reloaded, so returning here
  // finds it again instead of showing an idle form over a project it is still reading.
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
  const [isStrict, setIsStrict] = useState<boolean>(false);

  const onVerify = useCallback(async () => {
    if (!directory) {
      return;
    }

    log.info("Verifying:", { directory, isDltx, isStrict });

    await verifierService.verify(directory, isDltx, isStrict);
  }, [directory, isDltx, isStrict, log, verifierService]);

  useEffect(() => {
    verifierService.operation.reset();
  }, [directory, isDltx, isStrict, verifierService]);

  return (
    <JobPickerForm
      operation={verifierService.operation}
      isSubmitDisabled={!configs.isValid}
      title={"Verify LTX configs"}
      description={"Checks every LTX file in the directory. Nothing is written."}
      submitLabel={"Verify"}
      renderResult={(result) => <ConfigsVerifyResult result={result} />}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Configs directory"}
        description={"Directory of LTX files to validate"}
        field={configs}
      />

      <ConfigsDialectFormRow isDltx={isDltx} isDisabled={isRunning} onChange={setDltx} />

      <CheckboxFormRow
        label={"Strict"}
        description={"Also report what only a case-sensitive file system breaks on, such as an include in another case"}
        isChecked={isStrict}
        isDisabled={isRunning}
        onChange={setIsStrict}
      />
    </JobPickerForm>
  );
}
