import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveOrigins, ArchiveSourceUsage } from "@xrf/ipc/types/xrf-archive-stats";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { IStatBreakdownRow, StatFigure } from "@xrf/ui/stats";
import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/tools/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/tools/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";

export interface IArchiveOriginsSectionProps extends BaseComponentProps {
  origins: ArchiveOrigins;
  view: IArchiveStatisticsView;
}

/**
 * Where the files the engine would load actually come from.
 */
export function ArchiveOriginsSection({
  "data-testid": dataTestId = "archive-origins-section",
  id,
  className,
  origins,
  view,
}: IArchiveOriginsSectionProps): ReactElement {
  const rows: Array<IStatBreakdownRow> = useMemo(
    () =>
      origins.sources.map((usage: ArchiveSourceUsage) => ({
        files: usage.wins.files,
        id: usage.source,
        label: usage.source,
        sizeReal: usage.wins.sizeReal,
      })),
    [origins]
  );

  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Origins"}
      description={"Where the files the engine would load actually come from, in mount priority order."}
      rows={rows}
      view={view}
      isPreordered
    >
      <div className={"mb-4 flex flex-wrap gap-4"}>
        <StatFigure
          label={"Loose files"}
          value={formatCount(origins.loose.files)}
          hint={formatBytes(origins.loose.sizeReal)}
        />
        <StatFigure
          label={"Archived entries"}
          value={formatCount(origins.archived.files)}
          hint={formatBytes(origins.archived.sizeReal)}
        />
      </div>
    </ArchiveBreakdownSection>
  );
}
