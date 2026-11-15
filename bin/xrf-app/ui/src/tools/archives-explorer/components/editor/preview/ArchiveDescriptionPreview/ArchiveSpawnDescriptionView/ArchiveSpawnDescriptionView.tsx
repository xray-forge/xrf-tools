import { formatBytes } from "@xrf/format";
import { ArchiveSpawnDescription, ArchiveSpawnSection } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Spawn set"}>
        <EditorPanelProperty
          isInline
          label={"Objects"}
          value={`${description.objects}`}
          caption={`Across ${description.levels} ${description.levels === 1 ? "level" : "levels"}`}
        />

        <EditorPanelProperty isInline label={"Version"} value={`${description.version}`} />

        <EditorPanelProperty
          isInline
          label={"Identity"}
          value={description.guid}
          isMonospace
          caption={"What a save game is pinned to"}
        />

        <EditorPanelProperty
          isInline
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
          <EditorPanelProperty
            isInline
            key={section.id}
            label={formatSectionName(section)}
            value={formatBytes(section.size)}
            caption={describeSectionWeight(section, size)}
          />
        ))}
      </EditorPanelSection>
    </ReadingColumn>
  );
}
