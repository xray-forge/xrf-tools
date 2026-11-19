import { exists } from "@tauri-apps/plugin-fs";
import { useInjection } from "@wirestate/react";
import { assetsCommands } from "@xrf/ipc/commands/assets";
import { SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { XrayRootKind, XrayRootProbe } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";
import { EditorTextAction } from "@xrf/ui/editor/text-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { BuildIcon } from "@xrf/ui/icons/build";
import { ReactElement, useCallback } from "react";

import { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";
import { LEVEL_COMPILER_OBJECTS_CODEC } from "@/core/level/lib/compiler/level-compiler-objects";
import { EToolId } from "@/core/routing/tool";
import { THandOverPaths, useHandOverPaths } from "@/core/ui/form/use-hand-over-paths";
import {
  getLevelEditorBuildPaths,
  getLevelEditorGameRoot,
  ILevelEditorBuildPaths,
} from "@/tools/level-editor/lib/build/level-editor-build-paths";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/**
 * A game root's kind, which decides the directory the compilers read; null where the probe fails, so the root is taken
 * as game data.
 */
async function probeKind(root: string): Promise<Nullable<XrayRootKind>> {
  try {
    const probe: XrayRootProbe = await assetsCommands.probeRoot(root);

    return probe.kind;
  } catch {
    return null;
  }
}

/**
 * Opens the level compiler over the open scene, with the paths {@link getLevelEditorBuildPaths} lays out.
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

    const root: Nullable<string> = getLevelEditorGameRoot(scene);
    const paths: ILevelEditorBuildPaths = getLevelEditorBuildPaths(scene, root === null ? null : await probeKind(root));
    // Emptied where the decompile made no textures root: a missing path would block the form, and one remembered from
    // another scene would lend it that scene's strips.
    const isCut: boolean = await exists(paths.textures).catch(() => false);

    handOverPaths(EToolId.LEVEL_COMPILER, {
      [ELevelCompilerField.SOURCE]: paths.source,
      [ELevelCompilerField.DESTINATION]: paths.destination,
      [ELevelCompilerField.TEXTURES]: isCut ? paths.textures : null,
      // Empty roots make the build read the objects beside the scene.
      [ELevelCompilerField.OBJECTS]: LEVEL_COMPILER_OBJECTS_CODEC.write(paths.objects),
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
