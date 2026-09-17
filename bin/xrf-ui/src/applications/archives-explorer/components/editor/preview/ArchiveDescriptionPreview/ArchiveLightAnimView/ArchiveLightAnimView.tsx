import { Typography } from "@mui/material";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveLightAnimDescription, ArchiveLightAnimItem } from "@/core/ipc/types/xrf-app";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { filterByName, formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeTiming } from "./ArchiveLightAnimView.utils";

interface IArchiveLightAnimViewProps extends BaseComponentProps {
  description: ArchiveLightAnimDescription;
}

/**
 * The colour animation library: everything in the game that pulses, and how long each pulse takes.
 */
export function ArchiveLightAnimView({
  "data-testid": dataTestId = "archive-light-anim-view",
  id,
  className,
  description,
}: IArchiveLightAnimViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { items } = description;

  const matched: Array<ArchiveLightAnimItem> = useMemo(
    () => filterByName(items, filter, (item: ArchiveLightAnimItem) => item.name),
    [items, filter]
  );

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Colour animations"} isFirst>
        <ArchiveDescriptionRow
          label={"Animations"}
          value={`${items.length}`}
          caption={`Over ${formatCount(description.keys)} keyed colours`}
        />

        <ArchiveDescriptionRow
          label={"Version"}
          value={`${description.version}`}
          caption={
            description.isBgr
              ? "Colours are stored channel-swapped at this version; the engine corrects them as it loads them"
              : "Colours are stored as the engine uses them"
          }
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={filter.trim() ? `Animations (${matched.length} of ${items.length})` : `Animations (${items.length})`}
        caption={"A light, a glow or a particle effect names one of these rather than carrying its own colours"}
      >
        <EditorFilterInput
          className={"mb-2"}
          ariaLabel={"Filter animations"}
          query={filter}
          placeholder={"Filter animations"}
          onQueryChange={setFilter}
        />

        {matched.length ? (
          matched.map((item: ArchiveLightAnimItem) => (
            <ArchiveDescriptionRow
              key={item.name}
              label={item.name}
              value={describeTiming(item)}
              caption={`${formatCount(item.keys)} ${item.keys === 1 ? "key" : "keys"}`}
            />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No animation of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
