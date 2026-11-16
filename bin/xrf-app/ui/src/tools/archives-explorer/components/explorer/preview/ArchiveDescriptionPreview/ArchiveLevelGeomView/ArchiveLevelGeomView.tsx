import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveLevelGeomDescription, ArchiveLevelGeomLayout } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { describeLayout, describeLayoutShare, describeProgressiveMeshes } from "./ArchiveLevelGeomView.utils";

interface IArchiveLevelGeomViewProps extends BaseComponentProps {
  description: ArchiveLevelGeomDescription;
}

/**
 * A level's render geometry: what it costs to draw, and the vertex layouts it is built from.
 */
export function ArchiveLevelGeomView({
  "data-testid": dataTestId = "archive-level-geom-view",
  id,
  className,
  description,
}: IArchiveLevelGeomViewProps): ReactElement {
  const { layouts } = description;

  return (
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={description.isDetail ? "Detail geometry" : "Render geometry"}>
        <EditorPanelProperty
          isInline
          label={"Triangles"}
          value={formatCount(description.triangles)}
          caption={`Over ${formatCount(description.vertices)} vertices`}
        />

        <EditorPanelProperty
          isInline
          label={"Buffers"}
          value={`${description.vertexBuffers} vertex · ${description.indexBuffers} index`}
          caption={"What the renderer binds; a level is drawn out of a handful of large buffers"}
        />

        <EditorPanelProperty
          isInline
          label={"Progressive meshes"}
          value={formatCount(description.progressiveMeshes)}
          caption={describeProgressiveMeshes(description)}
        />

        <EditorPanelProperty
          isInline
          label={"Size"}
          value={formatBytes(description.size)}
          caption={"The vertices and indices are stepped over; only the shape is read"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Vertex layouts (${layouts.length})`}
        caption={"Grouped by what a vertex is made of, widest first"}
      >
        {layouts.map((layout: ArchiveLevelGeomLayout) => (
          <EditorPanelProperty
            isInline
            key={`${layout.stride}-${layout.elements}`}
            label={describeLayout(layout)}
            value={describeLayoutShare(layout)}
          />
        ))}
      </EditorPanelSection>
    </ReadingColumn>
  );
}
