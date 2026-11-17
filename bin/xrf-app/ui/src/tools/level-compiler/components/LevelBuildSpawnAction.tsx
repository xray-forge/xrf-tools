import { Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { ReactElement, useCallback } from "react";

import { EToolId } from "@/core/routing/tool";
import { ESpawnCompilerField } from "@/core/spawn/lib/compiler/spawn-compiler-field";
import { THandOverPaths, useHandOverPaths } from "@/core/ui/form/use-hand-over-paths";
import { getSpawnHandOverPaths, ISpawnHandOverPaths } from "@/tools/level-compiler/lib/spawn-hand-over-paths";

interface ILevelBuildSpawnActionProps extends BaseComponentProps {
  /** The level directory the build wrote; none disables the step. */
  level: Nullable<string>;
  /** The game data the compile read, whose level list names the level. */
  gamedata: Nullable<string>;
}

/**
 * The step from a built level to its spawn: the spawn compiler opened holding the directory the level was built into,
 * where its neighbours are built too, a `spawns` directory beside it, and the game data the level was compiled with.
 */
export function LevelBuildSpawnAction({
  "data-testid": dataTestId = "level-build-spawn-action",
  id,
  className,
  level,
  gamedata,
}: ILevelBuildSpawnActionProps): ReactElement {
  const handOverPaths: THandOverPaths = useHandOverPaths();

  const onBuildSpawn = useCallback(() => {
    if (!level) {
      return;
    }

    const paths: ISpawnHandOverPaths = getSpawnHandOverPaths(level, gamedata);

    handOverPaths(EToolId.SPAWN_COMPILER, {
      [ESpawnCompilerField.LEVELS_ROOT]: paths.levelsRoot,
      [ESpawnCompilerField.DESTINATION]: paths.destination,
      ...(paths.gamedata ? { [ESpawnCompilerField.GAMEDATA]: paths.gamedata } : {}),
    });
  }, [gamedata, handOverPaths, level]);

  return (
    <Button
      data-testid={dataTestId}
      id={id}
      className={className}
      size={"small"}
      variant={"text"}
      disabled={!level}
      startIcon={<AccountTreeIcon fontSize={"small"} />}
      onClick={onBuildSpawn}
    >
      Build spawn
    </Button>
  );
}
