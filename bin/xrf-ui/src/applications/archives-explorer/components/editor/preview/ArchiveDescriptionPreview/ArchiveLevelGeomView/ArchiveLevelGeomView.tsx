import { ReactElement } from "react";

import { ArchiveLevelGeomDescription, ArchiveLevelGeomLayout } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
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
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={description.isDetail ? "Detail geometry" : "Render geometry"} isFirst>
        <ArchiveDescriptionRow
          label={"Triangles"}
          value={formatCount(description.triangles)}
          caption={`Over ${formatCount(description.vertices)} vertices`}
        />

        <ArchiveDescriptionRow
          label={"Buffers"}
          value={`${description.vertexBuffers} vertex · ${description.indexBuffers} index`}
          caption={"What the renderer binds; a level is drawn out of a handful of large buffers"}
        />

        <ArchiveDescriptionRow
          label={"Progressive meshes"}
          value={formatCount(description.progressiveMeshes)}
          caption={describeProgressiveMeshes(description)}
        />

        <ArchiveDescriptionRow
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
          <ArchiveDescriptionRow
            key={`${layout.stride}-${layout.elements}`}
            label={describeLayout(layout)}
            value={describeLayoutShare(layout)}
          />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
