import { exists } from "@tauri-apps/plugin-fs";
import { useInjection } from "@wirestate/react";
import { SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorTextAction } from "@xrf/ui/editor/text-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { BuildIcon } from "@xrf/ui/icons/build";
import { ReactElement, useCallback } from "react";

import { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";
import { EToolId } from "@/core/routing/tool";
import { THandOverPaths, useHandOverPaths } from "@/core/ui/form/use-hand-over-paths";
import {
  getLevelEditorBuildPaths,
  ILevelEditorBuildPaths,
} from "@/tools/level-editor/lib/build/level-editor-build-paths";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/**
 * The step from the scene the editor shows to its build: the level compiler opened holding the scene, the level
 * directory beside its `rawdata`, the textures its decompile cut where they exist, and the game data it is drawn with.
 */
export function LevelEditorBuildAction({
  "data-testid": dataTestId = "level-editor-build-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);
  const handOverPaths: THandOverPaths = useHandOverPaths();

  const scene: Nullable<SelectedSceneDescription> = sceneService.scene.value;

  const onBuild = useCallback(async () => {
    if (!scene) {
      return;
    }

    const paths: ILevelEditorBuildPaths = getLevelEditorBuildPaths(scene);
    // A textures root the decompile never made would hold the form back as a path that does not exist, and one
    // remembered from another scene would lend this one that scene's strips, so the field is emptied instead.
    const isCut: boolean = await exists(paths.textures).catch(() => false);

    handOverPaths(EToolId.LEVEL_COMPILER, {
      [ELevelCompilerField.SOURCE]: paths.source,
      [ELevelCompilerField.DESTINATION]: paths.destination,
      [ELevelCompilerField.TEXTURES]: isCut ? paths.textures : null,
      ...(paths.gamedata ? { [ELevelCompilerField.GAMEDATA]: paths.gamedata } : {}),
      ...(paths.gameMaterials ? { [ELevelCompilerField.GAME_MATERIALS]: paths.gameMaterials } : {}),
    });
  }, [handOverPaths, scene]);

  return (
    <EditorTextAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Build"}
      description={
        scene
          ? "Open the level compiler holding this scene, its level directory in the gamedata beside its rawdata"
          : "Open a scene to build it"
      }
      icon={<BuildIcon />}
      isDisabled={!scene}
      onClick={onBuild}
    />
  );
}
