import { Checkbox, FormControlLabel } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { GamedataVerifyResult } from "@/applications/gamedata-verifier/components/GamedataVerifyResult";
import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function GamedataVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const pathsService: PathsService = useInjection(PathsService);
  const verifierService: GamedataVerifierService = useInjection(GamedataVerifierService);

  const [isStrict, setIsStrict] = useState<boolean>(false);

  // The run rather than this view's own flag: a full pass takes minutes and survives the window being reloaded, so
  // returning here finds it again instead of offering to start a second one.
  const job: Nullable<IJobState> = verifierService.job;
  const isRunning: boolean = Boolean(job);

  const gamedata: IPathField = usePathField({
    application: EApplicationId.GAMEDATA_VERIFIER,
    id: "gamedata",
    title: "Select gamedata directory",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolveExistingPathRole(EPathRole.GAMEDATA, pathsService.paths),
  });

  const root: Nullable<string> = gamedata.value;

  const onVerify = useCallback(async () => {
    if (!root) {
      return;
    }

    log.info("Verifying gamedata:", root);

    await verifierService.verify(root, isStrict);
  }, [isStrict, log, root, verifierService]);

  const onCancel = useCallback(() => verifierService.cancel(), [verifierService]);

  const onStrictChanged = useCallback(
    (_: ChangeEvent<HTMLInputElement>, checked: boolean) => {
      verifierService.reset();
      setIsStrict(checked);
    },
    [verifierService]
  );

  // A different tree invalidates whatever the previous run reported.
  useEffect(() => {
    verifierService.reset();
  }, [root, verifierService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!gamedata.isValid}
      title={"Verify gamedata"}
      description={"Runs every check over a gamedata tree: configs, meshes, textures, sounds, scripts and the rest."}
      error={verifierService.error ?? undefined}
      submitLabel={"Verify"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={verifierService.result ? <GamedataVerifyResult result={verifierService.result} /> : null}
      onSubmit={onVerify}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Gamedata"}
        description={"Directory holding configs, meshes, textures and the rest"}
        field={gamedata}
      />

      <FormControlLabel
        control={<Checkbox disabled={isRunning} checked={isStrict} onChange={onStrictChanged} />}
        label={"Strict"}
      />
    </PickerForm>
  );
}
