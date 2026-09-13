import { Chip } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { IArchiveStatisticsView } from "@/applications/archives-explorer/components/editor/statistics/archive-statistics-section";
import { ArchiveBreakdownSection } from "@/applications/archives-explorer/components/editor/statistics/ArchiveBreakdownSection";
import { ArchiveExtensionUsage } from "@/core/ipc/types/xrf-archive-stats";
import { IStatBreakdownRow } from "@/core/ui/stats/StatBreakdownTable";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IArchiveExtensionsSectionProps extends BaseComponentProps {
  extensions: Array<ArchiveExtensionUsage>;
  view: IArchiveStatisticsView;
}

/**
 * How much of the subject each file extension accounts for, as the spellings are actually written on disk.
 */
export function ArchiveExtensionsSection({
  "data-testid": dataTestId = "archive-extensions-section",
  id,
  className,
  extensions,
  view,
}: IArchiveExtensionsSectionProps): ReactElement {
  const undeclaredSpelling = useMemo(
    () => <Chip label={"unknown"} size={"small"} color={"warning"} variant={"outlined"} sx={{ height: 16 }} />,
    []
  );

  // Memoized because the table keys its totals and its ordering on this array's identity: a fresh one per render would
  // recompute both for nothing.
  const rows: Array<IStatBreakdownRow> = useMemo(
    () =>
      extensions.map((usage: ArchiveExtensionUsage) => ({
        files: usage.measure.files,
        id: usage.extension ?? "",
        label: usage.extension ?? "(no extension)",
        // Nothing to recognize is not the same as something unrecognized, so a nameless row is never marked.
        note: usage.extension !== null && !usage.isDeclared ? undeclaredSpelling : undefined,
        sizeReal: usage.measure.sizeReal,
      })),
    [extensions, undeclaredSpelling]
  );

  return (
    <ArchiveBreakdownSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Extensions"}
      description={
        "Every spelling found on disk. One marked unknown is a format the tooling does not declare, which is worth knowing about."
      }
      fact={`${extensions.length} spellings`}
      rows={rows}
      view={view}
    />
  );
}
