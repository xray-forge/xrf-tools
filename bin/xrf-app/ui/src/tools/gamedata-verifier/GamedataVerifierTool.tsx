import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { EngineChoiceFormRow } from "@/core/engine-target/components/EngineChoiceFormRow";
import { IEngineChoice, useEngineChoice } from "@/core/engine-target/lib/use-engine-choice";
import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { IGamedataVerification } from "@/tools/gamedata-verifier/lib/gamedata-verification";
import { GamedataVerifierService } from "@/tools/gamedata-verifier/services/gamedata-verifier.service";

import { GamedataVerifyResult } from "./components/GamedataVerifyResult";

export function GamedataVerifierTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const verifierService: GamedataVerifierService = useInjection(GamedataVerifierService);

  const [isStrict, setIsStrict] = useState<boolean>(false);

  // The run rather than this view's own flag: a full pass takes minutes and survives the window being reloaded, so
  // returning here finds it again instead of offering to start a second one.
  const isRunning: boolean = verifierService.operation.isRunning;

  const gamedata: IPathField = usePathField({
    tool: EToolId.GAMEDATA_VERIFIER,
    id: "gamedata",
    title: "Select gamedata directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const root: Nullable<string> = gamedata.value;
  const engine: IEngineChoice = useEngineChoice(gamedata.isValid ? root : null);

  const request: Nullable<IGamedataVerification> = useMemo(
    () => (root ? { engine: engine.choice, isStrict, root } : null),
    [engine.choice, isStrict, root]
  );

  const onVerify = useCallback(
    async (submitted: IGamedataVerification) => {
      log.info("Verifying gamedata:", submitted.root);

      await verifierService.verify(submitted);
    },
    [log, verifierService]
  );

  return (
    <JobPickerForm
      operation={verifierService.operation}
      request={request}
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
