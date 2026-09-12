import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

import { GamedataVerifyResult } from "./components/GamedataVerifyResult";

export function GamedataVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const verifierService: GamedataVerifierService = useInjection(GamedataVerifierService);

  const [isStrict, setIsStrict] = useState<boolean>(false);

  // The run rather than this view's own flag: a full pass takes minutes and survives the window being reloaded, so
  // returning here finds it again instead of offering to start a second one.
  const job: Nullable<IJobState> = verifierService.operation.job;
  const isRunning: boolean = verifierService.operation.isRunning;

  const gamedata: IPathField = usePathField({
    application: EApplicationId.GAMEDATA_VERIFIER,
    id: "gamedata",
    title: "Select gamedata directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const root: Nullable<string> = gamedata.value;

  const onVerify = useCallback(async () => {
    if (!root) {
      return;
    }

    log.info("Verifying gamedata:", root);

    await verifierService.verify(root, isStrict);
  }, [isStrict, log, root, verifierService]);

  const onCancel = useCallback(() => verifierService.operation.cancel(), [verifierService]);

  useEffect(() => {
    verifierService.operation.reset();
  }, [root, isStrict, verifierService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!gamedata.isValid}
      title={"Verify gamedata"}
      description={"Runs every check over a gamedata tree: configs, meshes, textures, sounds, scripts and the rest."}
      error={verifierService.operation.error ?? undefined}
      submitLabel={"Verify"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={
        verifierService.operation.result ? <GamedataVerifyResult result={verifierService.operation.result} /> : null
      }
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Gamedata"}
        description={"Directory holding configs, meshes, textures and the rest"}
        field={gamedata}
      />

      <CheckboxFormRow
        label={"Strict"}
        description={"Fully decode sounds and fail on missing bump companions or ineffective bump declarations"}
        isChecked={isStrict}
        isDisabled={isRunning}
        onChange={setIsStrict}
      />
    </PickerForm>
  );
}
