import { useInjection } from "@wirestate/react";
import { ABSENT_VALUE, formatBytes, formatCount, formatDegrees, formatNumber } from "@xrf/format";
import {
  ScenePartFileDescription,
  SceneToolDescription,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { XrayRoot } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";
import {
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
  EditorPanelSectionEmpty,
} from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import {
  formatSceneFacing,
  formatSceneMetres,
  formatScenePoint,
} from "@/tools/level-editor/lib/inspector/scene-object-format";
import { getSceneToolLabel } from "@/tools/level-editor/lib/tools/scene-tools";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

/** A tool's part files, one a line, each with its size and, where it is not the scene's, the GUID it carries. */
function describeToolFiles(tool: SceneToolDescription, guid: string): string {
  return tool.files
    .map((file: ScenePartFileDescription) => {
      const size: string = `${file.name} · ${formatBytes(file.size)}`;

      if (file.guid === guid) {
        return size;
      }

      return file.guid ? `${size} · GUID ${file.guid}` : `${size} · not a part`;
    })
    .join("\n");
}

/**
 * What the open scene's `.level` holds worth showing - where it lies, its GUID, where its library objects are looked
 * for and its textures read, what it builds with and where the editor's camera stood - and every tool's part files.
 */
export function LevelEditorScenePanel({
  "data-testid": dataTestId = "level-editor-scene-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const sceneService: LevelEditorSceneService = useInjection(LevelEditorSceneService);

  const scene: Nullable<SelectedSceneDescription> = sceneService.scene.value;

  if (!scene) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Scene"}>
        <EditorPanelEmpty label={"No scene open. Open a .level to see what it holds."} />
      </EditorPanel>
    );
  }

  const { buildParams, camera, tools } = scene;
  const size: number = tools.reduce((sum: number, tool: SceneToolDescription) => sum + tool.size, 0);

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Scene"}>
      <EditorPanelSection title={scene.name}>
        <EditorPanelProperty label={"Path"} value={scene.source.path} isMonospace />
        <EditorPanelProperty label={"GUID"} value={scene.guid} isMonospace />
        <EditorPanelProperty
          label={"Objects roots"}
          value={scene.objectsRoots.roots.map((root: XrayRoot) => root.path).join("\n") || "None"}
          isMonospace
        />
        <EditorPanelProperty
          label={"Game roots"}
          value={scene.game.roots.roots.map((root: XrayRoot) => root.path).join("\n") || "None: textures are missing"}
          isMonospace
        />
      </EditorPanelSection>

      <EditorPanelSection title={"Build"}>
        {buildParams ? (
          <>
            <EditorPanelProperty
              label={"Smoothing angle"}
              value={buildParams.smoothAngle === null ? ABSENT_VALUE : formatDegrees(buildParams.smoothAngle, 1)}
            />
            <EditorPanelProperty label={"Weld distance"} value={formatSceneMetres(buildParams.weldDistance)} />
            <EditorPanelProperty
              label={"Light pixels per metre"}
              value={formatNumber(buildParams.lightPixelPerMeter, 2)}
            />
            <EditorPanelProperty label={"Light jitter samples"} value={buildParams.lightJitterSamples} />
            <EditorPanelProperty label={"Light quality"} value={buildParams.lightQuality} />
          </>
        ) : (
          <EditorPanelSectionEmpty label={"The scene's [build_params] do not read."} />
        )}
      </EditorPanelSection>

      <EditorPanelSection title={"Camera"}>
        {camera ? (
          <>
            <EditorPanelProperty label={"Position"} value={formatScenePoint(camera.position)} />
            <EditorPanelProperty label={"Facing"} value={formatSceneFacing(camera.rotation)} />
          </>
        ) : (
          <EditorPanelSectionEmpty label={"The scene's [camera] does not read."} />
        )}
      </EditorPanelSection>

      <EditorPanelSection
        title={"Tools"}
        caption={`${formatCount(tools.length)} tools, ${formatBytes(size)} of part files`}
      >
        {tools.map((tool: SceneToolDescription) => (
          <EditorPanelProperty
            key={tool.name}
            label={getSceneToolLabel(tool.kind, tool.name)}
            value={describeToolFiles(tool, scene.guid)}
            isMonospace
          />
        ))}
      </EditorPanelSection>
    </EditorPanel>
  );
}
