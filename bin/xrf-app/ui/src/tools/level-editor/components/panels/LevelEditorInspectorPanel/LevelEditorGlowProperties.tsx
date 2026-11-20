import { formatHex } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { formatSceneMetres } from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorGlowPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.GLOW>;
}

/**
 * How large a glow is and what it is drawn with.
 */
export function LevelEditorGlowProperties({ properties }: ILevelEditorGlowPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Glow"}>
      <EditorPanelProperty label={"Radius"} value={formatSceneMetres(properties.radius)} />
      <EditorPanelProperty label={"Shader"} value={properties.shader || "None"} isMonospace />
      <EditorPanelProperty label={"Texture"} value={properties.texture || "None"} isMonospace />
      <EditorPanelProperty label={"Glow flags"} value={formatHex(properties.flags)} isMonospace />
    </EditorPanelSection>
  );
}
