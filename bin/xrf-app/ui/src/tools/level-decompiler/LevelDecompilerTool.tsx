import { useInjection } from "@wirestate/react";
import { ELevelSurfaces, LevelSurfaces } from "@xrf/ipc/types/xrf-rawdata";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow, IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { StringListFormRow } from "@xrf/ui/form/string-list-row";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { LEVEL_SCENE_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { LevelDecompilerService } from "@/tools/level-decompiler/services/level-decompiler.service";

import { LevelDecompileResultView } from "./components/LevelDecompileResultView";

const SURFACE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<LevelSurfaces>> = [
  { value: ELevelSurfaces.INFER, label: "Inferred from the level" },
  { value: ELevelSurfaces.DEFAULTS, label: "Editor defaults" },
];

export function LevelDecompilerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const decompilerService: LevelDecompilerService = useInjection(LevelDecompilerService);

  const isRunning: boolean = decompilerService.operation.isRunning;

  const [surfaces, setSurfaces] = useState<LevelSurfaces>(ELevelSurfaces.INFER);
  const [libraries, setLibraries] = useState<Array<string>>([]);
  const [isForced, setIsForced] = useState<boolean>(false);

  const source: IPathField = usePathField({
    tool: EToolId.LEVEL_DECOMPILER,
    id: "source",
    title: "Select compiled level directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const destination: IPathField = usePathField({
    tool: EToolId.LEVEL_DECOMPILER,
    id: "destination",
    title: "Select scene file to write",
    filters: LEVEL_SCENE_FILE_FILTERS,
    isSave: true,
    isDisabled: isRunning,
  });

  const objects: IPathField = usePathField({
    tool: EToolId.LEVEL_DECOMPILER,
    id: "objects",
    title: "Select objects root",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const textures: IPathField = usePathField({
    tool: EToolId.LEVEL_DECOMPILER,
    id: "textures",
    title: "Select game textures root",
    isDirectory: true,
    isRequired: false,
    isDisabled: isRunning,
  });

  const levelPath: Nullable<string> = source.value;
  const scenePath: Nullable<string> = destination.value;

  const onDecompileClicked = useCallback(async () => {
    if (!levelPath || !scenePath) {
      return;
    }

    log.info("Decompiling:", levelPath);

    await decompilerService.decompile({
      source: levelPath,
      destination: scenePath,
      objects: objects.value,
      textures: textures.value,
      rootSpawn: null,
      surfaces,
      gameMaterials: null,
      compilerShaders: null,
      libraries,
      isForced,
    });
  }, [decompilerService, isForced, levelPath, libraries, log, objects.value, scenePath, surfaces, textures.value]);

  // Any change of what would be asked invalidates the previous run's report.
  useEffect(() => {
    decompilerService.operation.reset();
  }, [decompilerService, isForced, levelPath, libraries, objects.value, scenePath, surfaces, textures.value]);

  return (
    <JobPickerForm
      operation={decompilerService.operation}
      isSubmitDisabled={!source.isValid || !destination.isValid || !objects.isValid || !textures.isValid}
      title={"Decompile a level"}
      description={"Writes a compiled level back as an SDK Level Editor scene, its objects and a provenance sidecar."}
      submitLabel={"Decompile"}
      renderResult={(result) => <LevelDecompileResultView result={result} outputPath={scenePath} />}
      onSubmit={onDecompileClicked}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Level"}
        description={"Compiled level directory, gamedata\\levels\\<level>"}
        field={source}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Scene"}
        description={"The .level to write, rawdata\\levels\\<level>.level; its parts go beside it"}
        field={destination}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Objects root"}
        description={"Where the scene's objects go; empty for rawdata\\objects beside the scene"}
        field={objects}
      />

      <PathFormRow
        isDisabled={isRunning}
        label={"Textures root"}
        description={"Where the trees' impostor strips go; empty for gamedata\\textures beside the rawdata"}
        field={textures}
      />

      <ChoiceFormRow<LevelSurfaces>
        label={"Surfaces"}
        description={"How each surface's game material and compiler shader are chosen"}
        options={SURFACE_OPTIONS}
        value={surfaces}
        isDisabled={isRunning}
        onChange={setSurfaces}
      />

      <StringListFormRow
        label={"Object libraries"}
        description={
          "Roots such as an SDK's rawdata\\objects, whose objects and trees found in the level are placed by reference"
        }
        values={libraries}
        addLabel={"Add library"}
        emptyLabel={"No library: details are left out"}
        placeholder={"C:\\SDK\\editors\\rawdata\\objects"}
        isDisabled={isRunning}
        onChange={setLibraries}
      />

      <CheckboxFormRow
        label={"Overwrite"}
        description={"Replace an existing scene, its objects and textures"}
        isChecked={isForced}
        isDisabled={isRunning}
        onChange={setIsForced}
      />
    </JobPickerForm>
  );
}
