import { formatHex } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorParticlePropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.PARTICLE>;
}

/**
 * The effect a particle object plays, and the game modes it plays in.
 */
export function LevelEditorParticleProperties({ properties }: ILevelEditorParticlePropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Particles"}>
      <EditorPanelProperty label={"Effect"} value={properties.reference} isMonospace />
      <EditorPanelProperty label={"Game types"} value={formatHex(properties.gameTypes)} isMonospace />
    </EditorPanelSection>
  );
}
