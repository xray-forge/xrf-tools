import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveFolderUsage } from "@/core/ipc/types/xrf-archive-stats";
import { IStatBreakdownRow } from "@/core/ui/stats/StatBreakdownTable";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IArchiveFoldersSectionProps extends BaseComponentProps {
  folders: Array<ArchiveFolderUsage>;
  view: IArchiveStatisticsView;
}

/** How much of the subject each top-level folder of the engine tree accounts for. */
export function ArchiveFoldersSection({
  "data-testid": dataTestId = "archive-folders-section",
  id,
  className,
  folders,
  view,
}: IArchiveFoldersSectionProps): ReactElement {
  const rows: Array<IStatBreakdownRow> = useMemo(
    () =>
      folders.map((usage: ArchiveFolderUsage) => ({
        files: usage.measure.files,
        id: usage.folder ?? "",
        // Trailing separator so a folder reads as one, and the root says what it is rather than showing nothing.
        label: usage.folder ? `${usage.folder}\\` : "(tree root)",
        sizeReal: usage.measure.sizeReal,
      })),
    [folders]
  );

  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Folders"}
      description={"Grouped by the engine tree's own top level, one segment deep."}
      fact={`${folders.length} folders`}
      rows={rows}
      view={view}
    />
  );
}
