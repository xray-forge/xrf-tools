import { useInjection } from "@wirestate/react";
import { ABSENT_VALUE, formatInstant } from "@xrf/format";
import { SelectedObjectBone, SelectedObjectDescription, SelectedObjectSurface } from "@xrf/ipc/types/xrf-app-visuals";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty, EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material/chip";
import { ReactElement } from "react";

import { EditorPanel } from "@/core/shell/chrome/EditorPanel";
import { VisualsService } from "@/tools/visuals-explorer/services/visuals";

import { describeObjectInference } from "./VisualObjectSourcePanel.utils";

/**
 * What an SDK object holds that the model compiled from it does not show: its surfaces and bones as authored, and where
 * a decompiled one came from and what was guessed in it.
 */
export function VisualObjectSourcePanel({
  "data-testid": dataTestId = "visual-object-source-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const object: Nullable<SelectedObjectDescription> = visualsService.object;

  if (!object) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Source"}>
        <EditorPanelEmpty label={"No SDK object open. Open a .object to see what it holds beyond its model."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Source"}>
      <EditorPanelSection title={"Object"} caption={"As authored, before the export splits it"}>
        <EditorPanelProperty label={"Meshes"} value={object.meshCount} />
        <EditorPanelProperty label={"Points"} value={object.pointCount} />
        <EditorPanelProperty label={"Faces"} value={object.faceCount} />
        <EditorPanelProperty label={"Smoothing groups"} value={object.smoothingGroupCount} />
        <EditorPanelProperty
          label={"Motion references"}
          value={object.motionReferences.length ? object.motionReferences.join(", ") : ABSENT_VALUE}
          isMonospace
        />
        <EditorPanelProperty label={"Own motions"} value={object.hasEmbeddedMotions ? "Yes" : "No"} />
        <EditorPanelProperty label={"Class script"} value={object.classScript ?? ABSENT_VALUE} isMonospace />
        <EditorPanelProperty label={"LODs"} value={object.lods ?? ABSENT_VALUE} isMonospace />
      </EditorPanelSection>

      {object.provenance ? (
        <EditorPanelSection title={"Decompiled"} caption={"Not authored: made from an OGF"}>
          <EditorPanelProperty label={"From"} value={object.provenance.source} isMonospace />
          <EditorPanelProperty label={"Tool"} value={`${object.provenance.tool} ${object.provenance.toolVersion}`} />
          <EditorPanelProperty label={"At"} value={formatInstant(object.provenance.decompiledAt * 1000)} />
          <EditorPanelProperty
            label={"Guessed rather than read"}
            value={
              object.provenance.inferred.length
                ? object.provenance.inferred.map(describeObjectInference).join("; ")
                : ABSENT_VALUE
            }
          />
        </EditorPanelSection>
      ) : null}

      {object.surfaces.map((surface: SelectedObjectSurface) => (
        <EditorPanelSection
          key={surface.name}
          title={
            <div className={"flex items-center gap-1.5"}>
              <span className={"min-w-0 break-all"}>{surface.name}</span>
              {surface.isTwoSided ? <Chip size={"small"} variant={"outlined"} label={"Two-sided"} /> : null}
            </div>
          }
          caption={`Surface, ${surface.faceCount} faces`}
        >
          <EditorPanelProperty label={"Texture"} value={surface.texture} isMonospace />
          <EditorPanelProperty label={"Shader"} value={surface.shader} isMonospace />
          <EditorPanelProperty label={"Compiler shader"} value={surface.compilerShader} isMonospace />
          <EditorPanelProperty label={"Game material"} value={surface.gameMaterial} isMonospace />
        </EditorPanelSection>
      ))}

      {object.bones.length ? (
        <EditorPanelSection title={"Bone physics"} caption={`${object.bones.length} bones`}>
          {object.bones.map((bone: SelectedObjectBone) => (
            <EditorPanelProperty
              key={bone.name}
              label={bone.parent ? `${bone.name} ← ${bone.parent}` : bone.name}
              value={[
                `${bone.shape} shape`,
                `${bone.joint} joint`,
                bone.gameMaterial,
                bone.mass === null ? null : `${bone.mass} kg`,
              ]
                .filter(Boolean)
                .join(", ")}
            />
          ))}
        </EditorPanelSection>
      ) : null}
    </EditorPanel>
  );
}
