import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveOrigins, ArchiveSourceUsage } from "@/core/ipc/types/xrf-archive-stats";
import { IStatBreakdownRow } from "@/core/ui/stats/StatBreakdownTable";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

export interface IArchiveOverridesSectionProps extends BaseComponentProps {
  origins: ArchiveOrigins;
  view: IArchiveStatisticsView;
}

/**
 * What the mount order hides, and what that costs.
 */
export function ArchiveOverridesSection({
  "data-testid": dataTestId = "archive-overrides-section",
  id,
  className,
  origins,
  view,
}: IArchiveOverridesSectionProps): ReactElement {
  const rows: Array<IStatBreakdownRow> = useMemo(
    () =>
      origins.sources.map((usage: ArchiveSourceUsage) => ({
        files: usage.hides.files,
        id: usage.source,
        label: usage.source,
        sizeReal: usage.hides.sizeReal,
      })),
    [origins]
  );

  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Overrides"}
      description={
        "Copies no lookup reaches, because a higher-priority source claims their path. Counted apart from the totals above."
      }
      fact={`${formatBytes(origins.hidden.sizeReal)} hidden`}
      rows={rows}
      view={view}
      isPreordered
    />
  );
}
