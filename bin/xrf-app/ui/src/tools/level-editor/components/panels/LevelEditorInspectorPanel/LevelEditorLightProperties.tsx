import { formatCount, formatDegrees, formatHex, formatNumber } from "@xrf/format";
import { ESceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { toDegrees } from "@xrf/math";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import {
  describeSceneLightType,
  formatSceneColor,
  formatSceneList,
  formatSceneMetres,
  formatScenePoint,
} from "@/tools/level-editor/lib/inspector/scene-object-format";
import { TSceneObjectPropertiesOf } from "@/tools/level-editor/lib/inspector/scene-object-kinds";

interface ILevelEditorLightPropertiesProps {
  properties: TSceneObjectPropertiesOf<ESceneObjectProperties.LIGHT>;
}

/**
 * What a light is, how bright and how far it lights, what it answers to and plays, and a fuzzy light's points.
 */
export function LevelEditorLightProperties({ properties }: ILevelEditorLightPropertiesProps): ReactElement {
  return (
    <EditorPanelSection title={"Light"} caption={describeSceneLightType(properties.lightType)}>
      <EditorPanelProperty label={"Colour"} value={formatSceneColor(properties.color)} isMonospace />
      <EditorPanelProperty label={"Brightness"} value={formatNumber(properties.brightness, 2)} />
      <EditorPanelProperty label={"Range"} value={formatSceneMetres(properties.range)} />
      <EditorPanelProperty label={"Attenuation"} value={formatSceneColor(properties.attenuation, 3)} isMonospace />

      {properties.cone === null ? null : (
        <EditorPanelProperty label={"Cone"} value={formatDegrees(toDegrees(properties.cone), 1)} />
      )}

      <EditorPanelProperty label={"Virtual size"} value={formatSceneMetres(properties.virtualSize)} />
      <EditorPanelProperty label={"Used in D3D"} value={properties.isUsedInD3d ? "Yes" : "No"} />
      <EditorPanelProperty label={"Control"} value={properties.control} />
      <EditorPanelProperty label={"Animation"} value={properties.animation || "None"} isMonospace />
      <EditorPanelProperty label={"Falloff texture"} value={properties.falloffTexture || "None"} isMonospace />
      <EditorPanelProperty label={"Light flags"} value={formatHex(properties.flags)} isMonospace />

      {properties.fuzzyPoints.length ? (
        <EditorPanelProperty
          label={`Fuzzy points (${formatCount(properties.fuzzyPoints.length)})`}
          value={formatSceneList(properties.fuzzyPoints, formatScenePoint)}
          isMonospace
        />
      ) : null}
    </EditorPanelSection>
  );
}
