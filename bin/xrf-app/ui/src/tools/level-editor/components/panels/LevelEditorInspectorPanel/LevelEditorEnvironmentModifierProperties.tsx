import { formatHex, formatNumber } from "@xrf/format";
import { ESceneSpawnKind } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { formatSceneMetres } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneSpawnKindOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

/** Digits a packed `0xAARRGGBB` colour is written with. */
const COLOR_DIGITS: number = 8;

interface ILevelEditorEnvironmentModifierPropertiesProps {
  spawn: TSceneSpawnKindOf<ESceneSpawnKind.ENVIRONMENT_MODIFIER>;
}

/**
 * How far and how strongly a local weather override reaches, and the values it mixes in.
 */
export function LevelEditorEnvironmentModifierProperties({
  spawn,
}: ILevelEditorEnvironmentModifierPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Spawn point"} caption={"Environment modifier"}>
      <EditorPanelProperty label={"Radius"} value={formatSceneMetres(spawn.radius)} />
      <EditorPanelProperty label={"Power"} value={formatNumber(spawn.power, 2)} />
      <EditorPanelProperty label={"View distance"} value={formatSceneMetres(spawn.viewDistance)} />
      <EditorPanelProperty label={"Fog colour"} value={formatHex(spawn.fogColor, COLOR_DIGITS)} isMonospace />
      <EditorPanelProperty label={"Fog density"} value={formatNumber(spawn.fogDensity, 3)} />
      <EditorPanelProperty label={"Ambient"} value={formatHex(spawn.ambientColor, COLOR_DIGITS)} isMonospace />
      <EditorPanelProperty label={"Sky"} value={formatHex(spawn.skyColor, COLOR_DIGITS)} isMonospace />
      <EditorPanelProperty label={"Hemisphere"} value={formatHex(spawn.hemiColor, COLOR_DIGITS)} isMonospace />
      <EditorPanelProperty label={"Mixes in"} value={formatHex(spawn.flags)} isMonospace />
    </EditorPanelSection>
  );
}
