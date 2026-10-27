import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveOverview } from "@xrf/ipc/types/xrf-archive-stats";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { StatFigure } from "@xrf/ui/stats";
import { ReactElement } from "react";

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
        <StatFigure label={"Files"} value={formatCount(overview.total.files)} />
        <StatFigure label={"Unpacked"} value={formatBytes(overview.total.sizeReal)} />
        <StatFigure label={"Sources"} value={formatCount(overview.sources)} />

        <StatFigure
          label={"Mean file"}
          value={formatBytes(overview.meanFile)}
          hint={`median ${formatBytes(overview.medianFile)}`}
        />

        <StatFigure label={"Largest file"} value={formatBytes(overview.largestFile)} />
        <StatFigure label={"Empty files"} value={formatCount(overview.emptyFiles)} />

        {overview.directories > 0 ? (
          <StatFigure label={"Directory entries"} value={formatCount(overview.directories)} hint={"hold no bytes"} />
        ) : null}
      </div>
    </DetailSection>
  );
}
