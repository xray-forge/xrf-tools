import { ESceneSpawnKind, SceneSpawnKind } from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive } from "@xrf/types";
import { ReactElement } from "react";

import { LevelEditorEntityProperties } from "./LevelEditorEntityProperties";
import { LevelEditorEnvironmentModifierProperties } from "./LevelEditorEnvironmentModifierProperties";
import { LevelEditorRespawnPointProperties } from "./LevelEditorRespawnPointProperties";

interface ILevelEditorSpawnKindPropertiesProps {
  spawn: SceneSpawnKind;
}

/**
 * What a spawn point places, by its kind.
 */
export function LevelEditorSpawnKindProperties({ spawn }: ILevelEditorSpawnKindPropertiesProps): ReactElement {
  switch (spawn.kind) {
    case ESceneSpawnKind.ENTITY:
      return <LevelEditorEntityProperties spawn={spawn} />;

    case ESceneSpawnKind.RESPAWN_POINT:
      return <LevelEditorRespawnPointProperties spawn={spawn} />;

    case ESceneSpawnKind.ENVIRONMENT_MODIFIER:
      return <LevelEditorEnvironmentModifierProperties spawn={spawn} />;

    default:
      return assertExhaustive(spawn);
  }
}
