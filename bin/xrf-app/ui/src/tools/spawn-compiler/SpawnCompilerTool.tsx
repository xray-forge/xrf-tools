import { useInjection } from "@wirestate/react";
import { SpawnListedLevel, SpawnRootRequest } from "@xrf/ipc/types/xrf-spawn-builder";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceListFormRow, IChoiceListFormRowOption } from "@xrf/ui/form/choice-list-row";
import { FormRow } from "@xrf/ui/form/row";
import { TextField } from "@xrf/ui/input";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { ESpawnCompilerField } from "@/core/spawn/lib/compiler/spawn-compiler-field";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { SpawnBuildResultView } from "@/tools/spawn-compiler/components/SpawnBuildResultView";
import { SpawnLevelRows } from "@/tools/spawn-compiler/components/SpawnLevelRows";
import {
  getSpawnBuildProblem,
  ISpawnBuildChoice,
  ISpawnBuildProblem,
  toSpawnBuildRequest,
} from "@/tools/spawn-compiler/lib/spawn-request";
import { SpawnCompilerService } from "@/tools/spawn-compiler/services/spawn-compiler.service";

/** The start choice standing for no start level: the actor starts where it stands. */
const ACTOR_START: string = "";

export function SpawnCompilerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const compilerService: SpawnCompilerService = useInjection(SpawnCompilerService);

  const isRunning: boolean = compilerService.operation.isRunning;

  const [chosen, setChosen] = useState<ReadonlyArray<string>>([]);
  const [start, setStart] = useState<string>(ACTOR_START);
  const [name, setName] = useState<string>("all");
  const [isSeparatorStrict, setIsSeparatorStrict] = useState<boolean>(false);
  const [isForced, setIsForced] = useState<boolean>(false);

  const gamedata: IPathField = usePathField({
    tool: EToolId.SPAWN_COMPILER,
    id: ESpawnCompilerField.GAMEDATA,
    title: "Select game data",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const levelsRoot: IPathField = usePathField({
    tool: EToolId.SPAWN_COMPILER,
    id: ESpawnCompilerField.LEVELS_ROOT,
    title: "Select levels directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.SPAWN_COMPILER,
    id: ESpawnCompilerField.DESTINATION,
    title: "Select spawn directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const listed: Nullable<Array<SpawnListedLevel>> = compilerService.levels.value;
  const built: Array<SpawnListedLevel> = useMemo(
    () => (listed ?? []).filter((level: SpawnListedLevel) => level.isComplete),
    [listed]
  );

  const startOptions: Array<IChoiceListFormRowOption<string>> = useMemo(
    () => [
      { value: ACTOR_START, label: "Where the actor stands", description: "The actor keeps its own level and place" },
      ...chosen.map((level: string) => ({
        value: level,
        label: level,
        description: "The actor moves to the first game vertex of this level a search from its own reaches",
      })),
    ],
    [chosen]
  );

  const choice: ISpawnBuildChoice = useMemo(
    () => ({
      gamedata: gamedata.value,
      levelsRoot: levelsRoot.value,
      destination: destination.value,
      levels: chosen,
      start: start === ACTOR_START ? null : start,
      name,
      isSeparatorStrict,
      isForced,
    }),
    [chosen, destination.value, gamedata.value, isForced, isSeparatorStrict, levelsRoot.value, name, start]
  );
  const problem: Nullable<ISpawnBuildProblem> = getSpawnBuildProblem(choice);
  const request: Nullable<SpawnRootRequest> = useMemo(() => toSpawnBuildRequest(choice), [choice]);

  const onBuild = useCallback(
    async (submitted: SpawnRootRequest) => {
      log.info("Building spawn:", submitted.name);

      await compilerService.build(submitted);
    },
    [compilerService, log]
  );

  const onLevelsChange = useCallback(
    (next: ReadonlyArray<string>) => {
      setChosen(next);

      if (start !== ACTOR_START && !next.includes(start)) {
        setStart(ACTOR_START);
      }
    },
    [start]
  );

  // The levels are listed again whenever either path changes, and a level no longer built there is unticked.
  useEffect(() => {
    if (!gamedata.value || !levelsRoot.value) {
      compilerService.resetLevels();

      return;
    }

    void compilerService.listLevels(gamedata.value, levelsRoot.value);
  }, [compilerService, gamedata.value, levelsRoot.value]);

  useEffect(() => {
    if (listed === null) {
      return;
    }

    function isBuilt(level: string): boolean {
      return built.some((it: SpawnListedLevel) => it.name === level);
    }

    setChosen((previous) => previous.filter(isBuilt));
    setStart((previous) => (previous === ACTOR_START || isBuilt(previous) ? previous : ACTOR_START));
  }, [built, listed]);

  return (
    <JobPickerForm
      operation={compilerService.operation}
      request={request}
      isSubmitDisabled={!gamedata.isValid || !levelsRoot.isValid || !destination.isValid}
      title={"Build a spawn"}
      description={
        "Builds a root spawn from built levels, as SDK 0.7's xrAI -s does: their AI maps, spawn lists and patrols " +
        "joined into one game graph, written as <name>.spawn."
      }
      submitLabel={"Build"}
      renderResult={(result) => <SpawnBuildResultView result={result} outputPath={destination.value} />}
      onSubmit={onBuild}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Game data"}
        description={"The game data holding configs\\game_levels.ltx, which names each level's id and offset"}
        field={gamedata}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Levels"}
        description={"The directory holding the built levels' directories, such as a gamedata's levels"}
        field={levelsRoot}
      />

      <FormRow
        label={"Levels built"}
        description={"The levels the spawn holds, each with its level.ai, level.spawn and level.game"}
        error={compilerService.levels.error?.message ?? (problem?.field === "levels" ? problem.message : null)}
        fact={
          compilerService.levels.isLoading
            ? "Listing the levels"
            : listed === null
              ? "Name the game data and the levels directory to list the levels"
              : built.length === 0
                ? "No level the game lists is built in this directory"
                : null
        }
        isGroup
      >
        <SpawnLevelRows levels={built} chosen={chosen} isDisabled={isRunning} onChange={onLevelsChange} />
      </FormRow>

      <ChoiceListFormRow
        label={"Start level"}
        description={problem?.field === "start" ? problem.message : "The level a new game starts on, xrAI's -start"}
        options={startOptions}
        value={start}
        isDisabled={isRunning}
        onChange={setStart}
      />

      <FormRow label={"Spawn name"} description={"Written as <name>.spawn; all is the game's own"}>
        {({ id, "aria-describedby": describedBy }) => (
          <TextField
            id={id}
            size={"small"}
            fullWidth={true}
            disabled={isRunning}
            value={name}
            error={problem?.field === "name"}
            helperText={problem?.field === "name" ? problem.message : undefined}
            slotProps={{ htmlInput: { "aria-describedby": describedBy } }}
            onChange={(event) => setName(event.target.value)}
          />
        )}
      </FormRow>

      <PathFormRow
        isDisabled={isRunning}
        label={"Spawn directory"}
        description={"The directory the spawn is written into, such as a gamedata's spawns"}
        field={destination}
      />

      <CheckboxFormRow
        label={"Strict separators"}
        description={"Refuse the build where a restrictor flagged for xrAI's separator check cuts an AI map apart"}
        isChecked={isSeparatorStrict}
        isDisabled={isRunning}
        onChange={setIsSeparatorStrict}
      />

      <CheckboxFormRow
        label={"Overwrite all.spawn"}
        description={"Write over the game's own all.spawn where the spawn directory holds it"}
        isChecked={isForced}
        isDisabled={isRunning}
        onChange={setIsForced}
      />
    </JobPickerForm>
  );
}
