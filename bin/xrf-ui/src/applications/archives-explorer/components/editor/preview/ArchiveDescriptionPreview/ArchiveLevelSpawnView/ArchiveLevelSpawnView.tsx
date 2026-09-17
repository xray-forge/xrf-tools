import { Typography } from "@mui/material";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveLevelSpawnDescription, ArchiveLevelSpawnSection } from "@/core/ipc/types/xrf-app";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { filterByName, formatCount, formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

interface IArchiveLevelSpawnViewProps extends BaseComponentProps {
  description: ArchiveLevelSpawnDescription;
}

/**
 * What a level spawns: the objects it plants, grouped by the section each is built from.
 */
export function ArchiveLevelSpawnView({
  "data-testid": dataTestId = "archive-level-spawn-view",
  id,
  className,
  description,
}: IArchiveLevelSpawnViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { sections } = description;

  const matched: Array<ArchiveLevelSpawnSection> = useMemo(
    () => filterByName(sections, filter, (section: ArchiveLevelSpawnSection) => section.name),
    [sections, filter]
  );

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Spawned objects"} isFirst>
        <ArchiveDescriptionRow
          label={"Objects"}
          value={formatCount(description.objects)}
          caption={`Out of ${sections.length} ${sections.length === 1 ? "section" : "sections"}`}
        />

        <ArchiveDescriptionRow
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of where the objects stand"}
        />

        <ArchiveDescriptionRow
          label={"Read by"}
          value={"xrServer::SLS_Default"}
          caption={"A flat run of spawn packets rather than a set: no header, no graph, no identity"}
        />

        <ArchiveDescriptionRow label={"Size"} value={formatBytes(description.size)} />
      </EditorPanelSection>

      <EditorPanelSection
        title={filter.trim() ? `Sections (${matched.length} of ${sections.length})` : `Sections (${sections.length})`}
        caption={"Most planted first; each names a config section rather than a file"}
      >
        <EditorFilterInput
          className={"mb-2"}
          ariaLabel={"Filter sections"}
          query={filter}
          placeholder={"Filter sections"}
          onQueryChange={setFilter}
        />

        {matched.length ? (
          matched.map((section: ArchiveLevelSpawnSection) => (
            <ArchiveDescriptionRow
              key={section.name}
              label={section.name}
              value={`${formatCount(section.objects)} ${section.objects === 1 ? "object" : "objects"}`}
            />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No section of this level is named that.
          </Typography>
        )}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
