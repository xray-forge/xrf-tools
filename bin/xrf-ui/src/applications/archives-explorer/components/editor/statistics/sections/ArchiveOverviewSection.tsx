import { ReactElement } from "react";

import { ArchiveOverview } from "@/core/ipc/types/xrf-archive-stats";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { StatFigure } from "@/core/ui/stats/StatFigure";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

export interface IArchiveOverviewSectionProps extends BaseComponentProps {
  overview: ArchiveOverview;
}

/** What the subject holds, before any breakdown of it. */
export function ArchiveOverviewSection({
  "data-testid": dataTestId = "archive-overview-section",
  id,
  className,
  overview,
}: IArchiveOverviewSectionProps): ReactElement {
  return (
    <DetailSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Overview"}
      description={"What this subject holds. Every breakdown below sums back to these totals."}
    >
      <div className={"flex flex-wrap gap-4"}>
        <StatFigure label={"Files"} value={overview.total.files.toLocaleString()} />
        <StatFigure label={"Unpacked"} value={formatBytes(overview.total.sizeReal)} />
        <StatFigure label={"Sources"} value={overview.sources.toLocaleString()} />

        <StatFigure
          label={"Mean file"}
          value={formatBytes(overview.meanFile)}
          hint={`median ${formatBytes(overview.medianFile)}`}
        />

        <StatFigure label={"Largest file"} value={formatBytes(overview.largestFile)} />
        <StatFigure label={"Empty files"} value={overview.emptyFiles.toLocaleString()} />

        {overview.directories > 0 ? (
          <StatFigure
            label={"Directory entries"}
            value={overview.directories.toLocaleString()}
            hint={"hold no bytes"}
          />
        ) : null}
      </div>
    </DetailSection>
  );
}
