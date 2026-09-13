import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveVolumeSummary } from "@/core/ipc/types/xrf-archive-stats";
import { IStatBreakdownRow } from "@/core/ui/stats/StatBreakdownTable";
import { inline } from "@/lib/callbacks/inline";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IArchiveVolumesSectionProps extends BaseComponentProps {
  volumes: Array<ArchiveVolumeSummary>;
  view: IArchiveStatisticsView;
}

/** Each volume as its own name table recorded it, before the merge. */
export function ArchiveVolumesSection({
  "data-testid": dataTestId = "archive-volumes-section",
  id,
  className,
  volumes,
  view,
}: IArchiveVolumesSectionProps): ReactElement {
  const rows: Array<IStatBreakdownRow> = useMemo(
    () =>
      volumes.map((volume: ArchiveVolumeSummary) => ({
        id: volume.path,
        // The volume's own entry count, which includes the directory records a merged listing drops.
        files: volume.entries,
        sizeReal: volume.sizeReal,
        label: inline(() => {
          const segments: Array<string> = volume.path.split(/[\\/]/);

          return segments[segments.length - 1] || volume.path;
        }),
      })),
    [volumes]
  );

  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Volumes"}
      description={
        "Each volume as its own name table recorded it, before the merge. Counts include the directory entries a merged listing drops."
      }
      fact={`${volumes.length} volumes`}
      rows={rows}
      view={view}
    />
  );
}
