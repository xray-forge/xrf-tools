import { ReactElement } from "react";

import { ArchiveSpawnDescription, ArchiveSpawnSection } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeSectionWeight, formatSectionName } from "./ArchiveSpawnDescriptionView.utils";

interface IArchiveSpawnDescriptionViewProps extends BaseComponentProps {
  description: ArchiveSpawnDescription;
}

/**
 * A spawn set, read from its header and the weight of its sections.
 */
export function ArchiveSpawnDescriptionView({
  "data-testid": dataTestId = "archive-spawn-description-view",
  id,
  className,
  description,
}: IArchiveSpawnDescriptionViewProps): ReactElement {
  const { sections, size } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Spawn set"} isFirst>
        <ArchiveDescriptionRow
          label={"Objects"}
          value={`${description.objects}`}
          caption={`Across ${description.levels} ${description.levels === 1 ? "level" : "levels"}`}
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />

        <ArchiveDescriptionRow
          label={"Identity"}
          value={description.guid}
          isMonospace
          caption={"What a save game is pinned to"}
        />

        <ArchiveDescriptionRow
          label={"Graph identity"}
          value={description.graphGuid}
          isMonospace
          caption={"The graph this set was built against"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={"Sections"}
        caption={"Read by their chunk headers alone; what they hold is not parsed"}
      >
        {sections.map((section: ArchiveSpawnSection) => (
          <ArchiveDescriptionRow
            key={section.id}
            label={formatSectionName(section)}
            value={formatBytes(section.size)}
            caption={describeSectionWeight(section, size)}
          />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
