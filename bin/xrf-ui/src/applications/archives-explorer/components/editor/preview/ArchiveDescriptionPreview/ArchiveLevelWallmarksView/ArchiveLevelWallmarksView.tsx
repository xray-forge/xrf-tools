import { ReactElement } from "react";

import {
  ArchiveDescribeScope,
  ArchiveLevelWallmarksDescription,
  ArchiveLevelWallmarkSlot,
} from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveLevelWallmarkSlotRow } from "./ArchiveLevelWallmarkSlotRow";

interface IArchiveLevelWallmarksViewProps extends BaseComponentProps {
  description: ArchiveLevelWallmarksDescription;
  scope: ArchiveDescribeScope;
}

/**
 * A level's baked decals: what the editor left on the walls, grouped by the material each is drawn with.
 */
export function ArchiveLevelWallmarksView({
  "data-testid": dataTestId = "archive-level-wallmarks-view",
  id,
  className,
  description,
  scope,
}: IArchiveLevelWallmarksViewProps): ReactElement {
  const { slots } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Baked decals"} isFirst>
        <ArchiveDescriptionRow
          label={"Decals"}
          value={formatCount(description.marks)}
          caption={`Over ${formatCount(description.vertices)} vertices, which is what the layer costs to draw`}
        />

        <ArchiveDescriptionRow
          label={"Materials"}
          value={`${slots.length}`}
          caption={"Each a blender and a texture the decals under it are drawn with"}
        />

        <ArchiveDescriptionRow
          label={"Read by"}
          value={"Nothing at play time"}
          caption={"The editor bakes these; the runtime places its own wallmarks as a session marks the walls"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Materials (${slots.length})`}
        caption={"In the order the file holds them, which is the order the exporter wrote them in"}
      >
        {slots.map((slot: ArchiveLevelWallmarkSlot, index: number) => (
          <ArchiveLevelWallmarkSlotRow key={index} slot={slot} scope={scope} />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
