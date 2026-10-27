import { ArchiveVolumeSummary } from "@xrf/ipc/types/xrf-archive-stats";
import { getPathName } from "@xrf/path";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { IStatBreakdownRow } from "@xrf/ui/stats";
import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/tools/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/tools/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";

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
        label: getPathName(volume.path) || volume.path,
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
