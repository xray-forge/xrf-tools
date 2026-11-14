import {
  ESceneObjectProperties,
  ESceneSpawnKind,
  SceneObjectProperties,
  SceneSpawnKind,
} from "@xrf/ipc/types/xrf-app-levels";

/** What one kind of scene object keeps beyond its name and transform. */
export type TSceneObjectPropertiesOf<K extends ESceneObjectProperties> = Extract<
  SceneObjectProperties,
  { kind: `${K}` }
>;

/** What one kind of spawn point places. */
export type TSceneSpawnKindOf<K extends ESceneSpawnKind> = Extract<SceneSpawnKind, { kind: `${K}` }>;
