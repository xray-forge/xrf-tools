import { useInjection } from "@wirestate/react";
import {
  DetailLightBackend,
  EDetailLightBackend,
  ELevelCompileStep,
  LevelCompileStep,
} from "@xrf/ipc/types/xrf-level-compiler";
import { ELevelBuildImpostorLimit } from "@xrf/ipc/types/xrf-rawdata";
import { RenderGpu } from "@xrf/ipc/types/xrf-renderer";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow, IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { StringListFormRow } from "@xrf/ui/form/string-list-row";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { LEVEL_SCENE_FILE_FILTERS, XR_LIBRARY_FILE_FILTERS } from "@/core/path/file-filters";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { EToolId } from "@/core/routing/tool";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import {
  COMPILE_STEPS,
  getCompileProblem,
  ICompileChoice,
  toCompileRequest,
} from "@/tools/level-compiler/lib/compile-request";
import { LevelCompilerService } from "@/tools/level-compiler/services/level-compiler.service";

import { LevelBuildResultView } from "./components/LevelBuildResultView";

const STEP_LABELS: Record<LevelCompileStep, { label: string; description: string }> = {
  [ELevelCompileStep.GEOMETRY]: {
    label: "Compile geometry",
    description: "The level, its geometry, collision and lightmaps from build.prj, with the SDK's xrLC",
  },
  [ELevelCompileStep.DETAILS]: {
    label: "Light details",
    description:
      "level.details lit from build.details, build.cform and build.lights, as xrDO_Light lights it, on the GPU or the CPU",
  },
  [ELevelCompileStep.AI]: {
    label: "Compile AI map",
    description: "level.ai from build.aimap, with the SDK's xrAI",
  },
};

const DETAIL_BACKENDS: ReadonlyArray<IChoiceFormRowOption<DetailLightBackend>> = [
  { label: "Auto", value: EDetailLightBackend.AUTO },
  { label: "GPU", value: EDetailLightBackend.GPU },
  { label: "CPU", value: EDetailLightBackend.CPU },
];

export function LevelCompilerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const compilerService: LevelCompilerService = useInjection(LevelCompilerService);
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const isRunning: boolean = compilerService.operation.isRunning;

  const [objects, setObjects] = useState<Array<string>>([]);
  const [isProofLighting, setIsProofLighting] = useState<boolean>(false);
  const [isSdkImpostorAtlas, setIsSdkImpostorAtlas] = useState<boolean>(false);
  const [isForced, setIsForced] = useState<boolean>(false);
  const [steps, setSteps] = useState<ReadonlySet<LevelCompileStep>>(new Set());
  const [detailBackend, setDetailBackend] = useState<DetailLightBackend>(EDetailLightBackend.AUTO);

  const source: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: "source",
    title: "Select scene file",
    filters: LEVEL_SCENE_FILE_FILTERS,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: "destination",
    title: "Select level directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const gameMaterials: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: "game-materials",
    title: "Select game materials",
    filters: XR_LIBRARY_FILE_FILTERS,
    isRequired: false,
    isDisabled: isRunning,
  });

  const textures: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: "textures",
    title: "Select game textures root",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const sdk: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: "sdk",
    title: "Select the SDK's editors directory",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const gamedata: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: "gamedata",
    title: "Select game data",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const scenePath: Nullable<string> = source.value;
  const levelPath: Nullable<string> = destination.value;
  // The build project needs the objects its placements name; the game files alone do not.
  const isMissingObjects: boolean = gameMaterials.value !== null && objects.length === 0;
  // The details trace on the GPU the renderer's settings choose, the automatic one where they choose none.
  const detailGpu: Nullable<RenderGpu> = renderSettingsService.renderGpu;
  const compile: ICompileChoice = useMemo(
    () => ({
      steps,
      sdk: sdk.value,
      gamedata: gamedata.value,
      textures: textures.value,
      detailBackend,
      detailGpu: detailGpu === null ? null : { vendor: detailGpu.vendor, device: detailGpu.device },
    }),
    [detailBackend, detailGpu, gamedata.value, sdk.value, steps, textures.value]
  );
  const compileProblem: Nullable<string> = getCompileProblem(compile);

  const onStepChanged = useCallback((step: LevelCompileStep, isChecked: boolean) => {
    setSteps((current: ReadonlySet<LevelCompileStep>) => {
      const next: Set<LevelCompileStep> = new Set(current);

      if (isChecked) {
        next.add(step);
      } else {
        next.delete(step);
      }

      return next;
    });
  }, []);

  const onBuildClicked = useCallback(async () => {
    if (!scenePath || !levelPath) {
      return;
    }

    log.info("Building:", scenePath);

    await compilerService.build(
      {
        source: scenePath,
        destination: levelPath,
        objects,
        gameMaterials: gameMaterials.value,
        textures: textures.value,
        isProofLighting,
        impostorLimit: isSdkImpostorAtlas ? ELevelBuildImpostorLimit.SDK : ELevelBuildImpostorLimit.EXTENDED,
        isForced,
      },
      toCompileRequest(compile)
    );
  }, [
    compile,
    compilerService,
    gameMaterials.value,
    isForced,
    isProofLighting,
    isSdkImpostorAtlas,
    levelPath,
    log,
    objects,
    scenePath,
    textures.value,
  ]);

  // Any change of what would be asked invalidates the previous run's report.
  useEffect(() => {
    compilerService.operation.reset();
  }, [
    compile,
    compilerService,
    gameMaterials.value,
    isForced,
    isProofLighting,
    isSdkImpostorAtlas,
    levelPath,
    objects,
    scenePath,
    textures.value,
  ]);

  return (
    <JobPickerForm
      operation={compilerService.operation}
      isSubmitDisabled={
        !source.isValid ||
        !destination.isValid ||
        !gameMaterials.isValid ||
        !textures.isValid ||
        !sdk.isValid ||
        !gamedata.isValid ||
        isMissingObjects ||
        compileProblem !== null
      }
      title={"Build a level"}
      description={
        "Writes a Level Editor scene's game files and, given its objects and game materials, its build.prj, then " +
        "compiles it with the steps chosen."
      }
      submitLabel={steps.size > 0 ? "Build and compile" : "Build"}
      renderResult={(result) => <LevelBuildResultView result={result} outputPath={levelPath} />}
      onSubmit={onBuildClicked}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Scene"}
        description={"The scene's .level; its parts are read from the directory beside it"}
        field={source}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Level"}
        description={"Level directory the files are written into"}
        field={destination}
      />

      <StringListFormRow
        label={"Objects roots"}
        description={"Where the scene's objects are read from, each from the first root holding it"}
        values={objects}
        addLabel={"Add root"}
        emptyLabel={isMissingObjects ? "The build project needs an objects root" : "No root: game files only"}
        placeholder={"C:\\SDK\\editors\\rawdata\\objects"}
        isDisabled={isRunning}
        onChange={setObjects}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Game materials"}
        description={"gamemtl.xr; without it only the game files are written, no build.prj"}
        field={gameMaterials}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Textures root"}
        description={"Where the details' textures and the trees' impostor strips are read from"}
        field={textures}
      />

      <CheckboxFormRow
        label={"Proof lighting"}
        description={"3 lightmap pixels a metre, 1 sample, quality 1 sky and sun: xrLC in minutes, not for shipping"}
        isChecked={isProofLighting}
        isDisabled={isRunning}
        onChange={setIsProofLighting}
      />

      <CheckboxFormRow
        label={"SDK impostor atlas"}
        description={"Keep the impostor atlas within SDK 0.7's 2048, refusing a level of more than 128 impostors"}
        isChecked={isSdkImpostorAtlas}
        isDisabled={isRunning}
        onChange={setIsSdkImpostorAtlas}
      />

      <CheckboxFormRow
        label={"Overwrite"}
        description={"Replace the files the build writes where the directory holds them"}
        isChecked={isForced}
        isDisabled={isRunning}
        onChange={setIsForced}
      />

      {COMPILE_STEPS.map((step: LevelCompileStep) => (
        <CheckboxFormRow
          key={step}
          label={STEP_LABELS[step].label}
          description={STEP_LABELS[step].description}
          isChecked={steps.has(step)}
          isDisabled={isRunning}
          onChange={(isChecked: boolean) => onStepChanged(step, isChecked)}
        />
      ))}

      <ChoiceFormRow
        label={"Details traced on"}
        description={
          "Auto: the GPU where it traces rays, else the CPU. The GPU is the one chosen in the renderer settings, " +
          "several times faster; the CPU is the reference, every slot within one step of the GPU's"
        }
        options={DETAIL_BACKENDS}
        value={detailBackend}
        isDisabled={isRunning || !steps.has(ELevelCompileStep.DETAILS)}
        onChange={setDetailBackend}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"SDK"}
        description={
          compileProblem ??
          "The SDK's editors directory, whose bin holds xrLC and xrAI; for the geometry and the AI map"
        }
        field={sdk}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Game data"}
        description={"Where the compilers read gamemtl.xr and shaders_xrlc.xr; the textures root above serves them too"}
        field={gamedata}
      />
    </JobPickerForm>
  );
}
