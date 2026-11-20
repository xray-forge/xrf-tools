import { formatHex } from "@xrf/format";
import { ESceneSpawnKind } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { TSceneSpawnKindOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorRespawnPointPropertiesProps {
  spawn: TSceneSpawnKindOf<ESceneSpawnKind.RESPAWN_POINT>;
}

/**
 * Where a multiplayer team, artefact or item respawns, and in which game modes.
 */
export function LevelEditorRespawnPointProperties({ spawn }: ILevelEditorRespawnPointPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Spawn point"} caption={"Respawn point"}>
      <EditorPanelProperty label={"Team"} value={spawn.team} />
      <EditorPanelProperty label={"Point type"} value={spawn.pointType} />
      <EditorPanelProperty label={"Game types"} value={formatHex(spawn.gameTypes)} isMonospace />
      <EditorPanelProperty label={"Profile"} value={spawn.profile || "None"} isMonospace />
    </EditorPanelSection>
  );
}
