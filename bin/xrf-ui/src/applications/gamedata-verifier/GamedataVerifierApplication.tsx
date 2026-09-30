import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

import { GamedataVerifyResult } from "./components/GamedataVerifyResult";

export function GamedataVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const verifierService: GamedataVerifierService = useInjection(GamedataVerifierService);
  const settingsService: SettingsService = useInjection(SettingsService);

  const [isStrict, setIsStrict] = useState<boolean>(false);

  // The run rather than this view's own flag: a full pass takes minutes and survives the window being reloaded, so
  // returning here finds it again instead of offering to start a second one.
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

    await verifierService.verify({ engine: settingsService.engine, isStrict, root });
  }, [isStrict, log, root, settingsService, verifierService]);

  useEffect(() => {
    verifierService.operation.reset();
  }, [root, isStrict, verifierService]);

  return (
    <JobPickerForm
      operation={verifierService.operation}
      isSubmitDisabled={!gamedata.isValid}
      title={"Verify gamedata"}
      description={"Runs every check over a gamedata tree: configs, meshes, textures, sounds, scripts and the rest."}
      submitLabel={"Verify"}
      renderResult={(result) => <GamedataVerifyResult result={result} />}
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
    </JobPickerForm>
  );
}
