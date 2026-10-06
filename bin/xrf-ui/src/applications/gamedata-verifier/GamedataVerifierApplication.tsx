import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import { EngineChoiceFormRow } from "@/core/engine-target/components";
import { IEngineChoice, useEngineChoice } from "@/core/engine-target/lib";
import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { EApplicationId } from "@/core/routing/application";
import { CheckboxFormRow, IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

import { GamedataVerifyResult } from "./components/GamedataVerifyResult";

export function GamedataVerifierApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const verifierService: GamedataVerifierService = useInjection(GamedataVerifierService);

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
  const engine: IEngineChoice = useEngineChoice(gamedata.isValid ? root : null);

  const onVerify = useCallback(async () => {
    if (!root) {
      return;
    }

    log.info("Verifying gamedata:", root);

    await verifierService.verify({ engine: engine.choice, isStrict, root });
  }, [engine.choice, isStrict, log, root, verifierService]);

  useEffect(() => {
    verifierService.operation.reset();
  }, [engine.choice, root, isStrict, verifierService]);

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

      <EngineChoiceFormRow engine={engine} isDisabled={isRunning} />
    </JobPickerForm>
  );
}
