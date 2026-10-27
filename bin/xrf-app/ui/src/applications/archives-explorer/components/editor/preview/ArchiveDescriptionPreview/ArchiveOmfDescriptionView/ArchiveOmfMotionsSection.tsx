import { formatCount } from "@xrf/format";
import { ArchiveOmfMotion } from "@xrf/ipc/types/xrf-app-archives";
import { filterByName } from "@xrf/search";
import { EditorFilterInput } from "@xrf/ui/editor/filter-input";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveOmfMotionRow } from "./ArchiveOmfMotionRow";

interface IArchiveOmfMotionsSectionProps extends BaseComponentProps {
  motions: Array<ArchiveOmfMotion>;
}

/**
 * Every motion the bank holds, in the order it declares them.
 */
export function ArchiveOmfMotionsSection({
  "data-testid": dataTestId = "archive-omf-motions-section",
  id,
  className,
  motions,
}: IArchiveOmfMotionsSectionProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const matched: Array<ArchiveOmfMotion> = useMemo(
    () => filterByName(motions, filter, (motion: ArchiveOmfMotion) => motion.name),
    [filter, motions]
  );

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={
        filter.trim()
          ? `Motions (${formatCount(matched.length)} of ${formatCount(motions.length)})`
          : `Motions (${formatCount(motions.length)})`
      }
      caption={"In the order the bank declares them, which is the order the engine pairs them by"}
    >
      <EditorFilterInput
        className={"mb-2"}
        query={filter}
        placeholder={"Filter motions"}
        ariaLabel={"Filter motions"}
        onQueryChange={setFilter}
      />

      {matched.length ? (
        matched.map((motion: ArchiveOmfMotion, index: number) => (
          <ArchiveOmfMotionRow key={`${index}-${motion.name}`} motion={motion} />
        ))
      ) : (
        <Typography className={"text-text-disabled"} variant={"body2"}>
          No motion of this bank is named that.
        </Typography>
      )}
    </EditorPanelSection>
  );
}
