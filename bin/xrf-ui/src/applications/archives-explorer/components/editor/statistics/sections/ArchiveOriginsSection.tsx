import { Box } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveOrigins, ArchiveSourceUsage } from "@/core/ipc/types/xrf-archive-stats";
import { IStatBreakdownRow } from "@/core/ui/stats/StatBreakdownTable";
import { StatFigure } from "@/core/ui/stats/StatFigure";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

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
      <Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, marginBottom: 2 }}>
        <StatFigure
          label={"Loose files"}
          value={origins.loose.files.toLocaleString()}
          hint={formatBytes(origins.loose.sizeReal)}
        />
        <StatFigure
          label={"Archived entries"}
          value={origins.archived.files.toLocaleString()}
          hint={formatBytes(origins.archived.sizeReal)}
        />
      </Box>
    </ArchiveBreakdownSection>
  );
}
