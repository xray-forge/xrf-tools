import { useInjection } from "@wirestate/react";
import { ELevelBuildImpostorLimit } from "@xrf/ipc/types/xrf-rawdata";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { StringListFormRow } from "@xrf/ui/form/string-list-row";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { LEVEL_SCENE_FILE_FILTERS, XR_LIBRARY_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { LevelCompilerService } from "@/tools/level-compiler/services/level-compiler.service";

import { LevelBuildResultView } from "./components/LevelBuildResultView";

export function LevelCompilerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const compilerService: LevelCompilerService = useInjection(LevelCompilerService);

  const isRunning: boolean = compilerService.operation.isRunning;

  const [objects, setObjects] = useState<Array<string>>([]);
  const [isProofLighting, setIsProofLighting] = useState<boolean>(false);
  const [isSdkImpostorAtlas, setIsSdkImpostorAtlas] = useState<boolean>(false);
  const [isForced, setIsForced] = useState<boolean>(false);

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

  const scenePath: Nullable<string> = source.value;
  const levelPath: Nullable<string> = destination.value;
  // The build project needs the objects its placements name; the game files alone do not.
  const isMissingObjects: boolean = gameMaterials.value !== null && objects.length === 0;

  const onBuildClicked = useCallback(async () => {
    if (!scenePath || !levelPath) {
      return;
    }

    log.info("Building:", scenePath);

    await compilerService.build({
      source: scenePath,
      destination: levelPath,
      objects,
      gameMaterials: gameMaterials.value,
      textures: textures.value,
      isProofLighting,
      impostorLimit: isSdkImpostorAtlas ? ELevelBuildImpostorLimit.SDK : ELevelBuildImpostorLimit.EXTENDED,
      isForced,
    });
  }, [
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
        !source.isValid || !destination.isValid || !gameMaterials.isValid || !textures.isValid || isMissingObjects
      }
      title={"Build a level"}
      description={"Writes a Level Editor scene's game files and, given its objects and game materials, its build.prj."}
      submitLabel={"Build"}
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
    </JobPickerForm>
  );
}
