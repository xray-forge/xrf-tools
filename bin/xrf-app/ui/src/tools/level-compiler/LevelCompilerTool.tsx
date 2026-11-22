import { useInjection } from "@wirestate/react";
import {
  AiMapCovers,
  EAiMapCovers,
  ELevelCompileStep,
  ELightQuality,
  ERayBackend,
  LevelCompileRequest,
  LevelCompileStep,
  LightQuality,
  RayBackend,
} from "@xrf/ipc/types/xrf-level-compiler";
import { ELevelBuildImpostorLimit, LevelBuildRequest } from "@xrf/ipc/types/xrf-rawdata";
import { RenderGpu } from "@xrf/ipc/types/xrf-renderer-core";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { StringListFormRow } from "@xrf/ui/form/string-list-row";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";
import { LEVEL_SCENE_FILE_FILTERS, XR_LIBRARY_FILE_FILTERS } from "@/core/path/file-filters";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { IPathField, PathFormRow, usePathField, useRememberedPaths } from "@/core/ui/form";
import {
  AI_COVERS_OPTIONS,
  LIGHT_QUALITY_OPTIONS,
  RAY_BACKEND_OPTIONS,
} from "@/tools/level-compiler/lib/compile-options";
import {
  getCompileProblem,
  getOfferedSteps,
  ICompileChoice,
  ICompileProblem,
  toCompileRequest,
} from "@/tools/level-compiler/lib/compile-request";
import { useSceneDetails } from "@/tools/level-compiler/lib/use-scene-details";
import { LevelCompilerService } from "@/tools/level-compiler/services/level-compiler.service";

import { LevelBuildResultView } from "./components/LevelBuildResultView";
import { LevelCompileStepRows } from "./components/LevelCompileStepRows";

/** A build and the compile run after it, as one job. */
interface ILevelCompilerRequest {
  build: LevelBuildRequest;
  /** None to build alone. */
  compile: Nullable<LevelCompileRequest>;
}

export function LevelCompilerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const compilerService: LevelCompilerService = useInjection(LevelCompilerService);
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const isRunning: boolean = compilerService.operation.isRunning;

  // Remembered, so a scene handed over from the editor arrives with the roots it was opened with.
  const [objects, setObjects] = useRememberedPaths({ tool: EToolId.LEVEL_COMPILER, id: ELevelCompilerField.OBJECTS });
  const [isSdkImpostorAtlas, setIsSdkImpostorAtlas] = useState<boolean>(false);
  const [isForced, setIsForced] = useState<boolean>(false);
  const [ticked, setTicked] = useState<ReadonlySet<LevelCompileStep>>(new Set());
  const [rayBackend, setRayBackend] = useState<RayBackend>(ERayBackend.AUTO);
  const [aiCovers, setAiCovers] = useState<AiMapCovers>(EAiMapCovers.FULL);
  const [lightQuality, setLightQuality] = useState<LightQuality>(ELightQuality.SCENE);

  const source: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: ELevelCompilerField.SOURCE,
    title: "Select scene file",
    filters: LEVEL_SCENE_FILE_FILTERS,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: ELevelCompilerField.DESTINATION,
    title: "Select level directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isRunning,
  });

  const gameMaterials: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: ELevelCompilerField.GAME_MATERIALS,
    title: "Select game materials",
    filters: XR_LIBRARY_FILE_FILTERS,
    isRequired: false,
    isDisabled: isRunning,
  });

  const textures: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: ELevelCompilerField.TEXTURES,
    title: "Select game textures root",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const gamedata: IPathField = usePathField({
    tool: EToolId.LEVEL_COMPILER,
    id: ELevelCompilerField.GAMEDATA,
    title: "Select game data",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const scenePath: Nullable<string> = source.value;
  const levelPath: Nullable<string> = destination.value;
  const hasDetails: Nullable<boolean> = useSceneDetails(scenePath);
  const steps: ReadonlySet<LevelCompileStep> = useMemo(() => getOfferedSteps(ticked, hasDetails), [hasDetails, ticked]);
  // The rays trace on the GPU the renderer's settings choose, the automatic one where they choose none.
  const rayGpu: Nullable<RenderGpu> = renderSettingsService.renderGpu;
  const isTracing: boolean =
    steps.has(ELevelCompileStep.LIGHT) ||
    steps.has(ELevelCompileStep.DETAILS) ||
    (steps.has(ELevelCompileStep.AI) && aiCovers === EAiMapCovers.FULL);
  const compile: ICompileChoice = useMemo(
    () => ({
      steps,
      gameMaterials: gameMaterials.value,
      gamedata: gamedata.value,
      textures: textures.value,
      rayBackend,
      rayGpu: rayGpu === null ? null : { vendor: rayGpu.vendor, device: rayGpu.device },
      aiCovers,
      lightQuality,
    }),
    [aiCovers, gameMaterials.value, gamedata.value, lightQuality, rayBackend, rayGpu, steps, textures.value]
  );
  const compileProblem: Nullable<ICompileProblem> = getCompileProblem(compile);

  function describeProblem(field: Nullable<ELevelCompilerField>, otherwise: string): string {
    return compileProblem !== null && compileProblem.field === field ? compileProblem.message : otherwise;
  }

  const request: Nullable<ILevelCompilerRequest> = useMemo(() => {
    if (!scenePath || !levelPath) {
      return null;
    }

    return {
      build: {
        source: scenePath,
        destination: levelPath,
        objects,
        gameMaterials: gameMaterials.value,
        textures: textures.value,
        impostorLimit: isSdkImpostorAtlas ? ELevelBuildImpostorLimit.SDK : ELevelBuildImpostorLimit.EXTENDED,
        isForced,
      },
      compile: toCompileRequest(compile),
    };
  }, [compile, gameMaterials.value, isForced, isSdkImpostorAtlas, levelPath, objects, scenePath, textures.value]);

  const onBuild = useCallback(
    async (submitted: ILevelCompilerRequest) => {
      log.info("Building:", submitted.build.source);

      await compilerService.build(submitted.build, submitted.compile);
    },
    [compilerService, log]
  );

  return (
    <JobPickerForm
      operation={compilerService.operation}
      request={request}
      isSubmitDisabled={
        !source.isValid ||
        !destination.isValid ||
        !gameMaterials.isValid ||
        !textures.isValid ||
        !gamedata.isValid ||
        compileProblem !== null
      }
      title={"Build a level"}
      description={
        "Writes a Level Editor scene's game files and, given its objects and game materials, its build.prj, then " +
        "compiles it with the steps chosen. A run the compile would refuse is refused before anything is written."
      }
      submitLabel={steps.size > 0 ? "Build and compile" : "Build"}
      renderResult={(result) => (
        <LevelBuildResultView result={result} outputPath={levelPath} gamedata={gamedata.value} />
      )}
      onSubmit={onBuild}
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
        description={
          "Where the scene's objects are read from, each from the first root holding it; none for objects beside the " +
          "scene's directory, where a decompile writes them"
        }
        values={objects}
        addLabel={"Add root"}
        emptyLabel={"None: objects beside the scene's directory"}
        placeholder={"C:\\SDK\\editors\\rawdata\\objects"}
        isDisabled={isRunning}
        onChange={setObjects}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Game materials"}
        description={describeProblem(
          ELevelCompilerField.GAME_MATERIALS,
          "gamemtl.xr, which build.prj and a placed object's ladders and breakables are made from; without it only " +
            "the game files of a scene placing no objects are written"
        )}
        field={gameMaterials}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Textures root"}
        description={
          "Where the details' textures and the impostor strips are read from, such as the one a decompile wrote; " +
          "the compile reads it before the game data's own textures"
        }
        field={textures}
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
        description={
          "Replace what the build and the compile write where the directory holds it, and remove what of their own " +
          "an earlier run wrote that this one does not"
        }
        isChecked={isForced}
        isDisabled={isRunning}
        onChange={setIsForced}
      />

      <LevelCompileStepRows steps={steps} hasDetails={hasDetails} isDisabled={isRunning} onChange={setTicked} />

      <ChoiceFormRow
        label={"Light quality"}
        description={
          "Scene: build.prj's own sky, sun, lightmap density and samples. Draft: a quick estimate, not for shipping, " +
          "with the sky and sun at quality 1, lightmaps of 3 texels a metre with one sample each, and each " +
          "multi-use object lit from one sample"
        }
        options={LIGHT_QUALITY_OPTIONS}
        value={lightQuality}
        isDisabled={isRunning || !steps.has(ELevelCompileStep.LIGHT)}
        onChange={setLightQuality}
      />

      <ChoiceFormRow
        label={"AI map covers"}
        description={
          "Full: traced against build.cform, as xrAI -f traces them. None: every cover 0, as xrAI -draft leaves " +
          "them; needs no compiled geometry"
        }
        options={AI_COVERS_OPTIONS}
        value={aiCovers}
        isDisabled={isRunning || !steps.has(ELevelCompileStep.AI)}
        onChange={setAiCovers}
      />

      <ChoiceFormRow
        label={"Rays traced on"}
        description={
          "The rays of the light, the details and the AI map's covers. Auto: the GPU where it supports ray tracing, " +
          "else the CPU. The GPU is the one chosen in the renderer settings and several times faster; the CPU gives " +
          "the reference result"
        }
        options={RAY_BACKEND_OPTIONS}
        value={rayBackend}
        isDisabled={isRunning || !isTracing}
        onChange={setRayBackend}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Game data"}
        description={describeProblem(
          ELevelCompilerField.GAMEDATA,
          // The steps chosen have no row of their own to say what they lack.
          describeProblem(
            null,
            "Where the compilers read gamemtl.xr, shaders_xrlc.xr and the textures from, after the textures root above"
          )
        )}
        field={gamedata}
      />
    </JobPickerForm>
  );
}
