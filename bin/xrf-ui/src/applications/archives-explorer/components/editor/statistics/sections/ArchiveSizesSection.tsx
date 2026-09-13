import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveSizeBand } from "@/core/ipc/types/xrf-archive-stats";
import { IStatBreakdownRow } from "@/core/ui/stats/StatBreakdownTable";
import { inline } from "@/lib/callbacks/inline";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

export interface IArchiveSizesSectionProps extends BaseComponentProps {
  sizes: Array<ArchiveSizeBand>;
  view: IArchiveStatisticsView;
}

/** Which size bands hold the files and the bytes. */
export function ArchiveSizesSection({
  "data-testid": dataTestId = "archive-sizes-section",
  id,
  className,
  sizes,
  view,
}: IArchiveSizesSectionProps): ReactElement {
  const rows: Array<IStatBreakdownRow> = useMemo(() => {
    return sizes.map((band: ArchiveSizeBand) => ({
      id: String(band.from),
      files: band.measure.files,
      sizeReal: band.measure.sizeReal,
      label: inline(() => {
        if (band.to === null) {
          return `${formatBytes(band.from)} and up`;
        }

        // The first band admits nothing but zero, so naming a range for it would overstate what it holds.
        if (band.from === 0) {
          return "Empty";
        }

        return `${formatBytes(band.from)} - ${formatBytes(band.to)}`;
      }),
    }));
  }, [sizes]);

  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Sizes"}
      description={
        "Which size bands hold the files and the bytes. Each row names the band it covers, so the order follows the measurement like every other breakdown."
      }
      rows={rows}
      view={view}
    />
  );
}
