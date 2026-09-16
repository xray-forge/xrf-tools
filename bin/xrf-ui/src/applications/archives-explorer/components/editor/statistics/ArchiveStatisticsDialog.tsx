import { default as QueryStatsIcon } from "@mui/icons-material/QueryStats";
import { Alert, CircularProgress, Dialog, DialogContent, List, ListItemButton, ListItemText } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useId, useMemo, useState } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { getWellFillSx } from "@/core/theme/surface";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { EStatMeasure } from "@/core/ui/stats/stat-measure";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable, Optional } from "@/lib/types/general";

import { EArchiveStatisticsSection, IArchiveStatisticsSection } from "./archive-statistics-section";
import { listArchiveStatisticsSections } from "./archive-statistics-sections";

export interface IArchiveStatisticsDialogProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * What the open subject holds, as a dialog of named sections.
 */
export function ArchiveStatisticsDialog({
  "data-testid": dataTestId = "archive-statistics-dialog",
  id,
  className,
  isOpen,
  onClose,
}: IArchiveStatisticsDialogProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const titleId: string = useId();

  const [selectedId, setSelectedId] = useState<EArchiveStatisticsSection>(EArchiveStatisticsSection.OVERVIEW);
  const [measure, setMeasure] = useState<EStatMeasure>(EStatMeasure.BYTES);

  const statistics: Nullable<ArchiveStatistics> = archivesService.statistics.value;
  const sections: ReadonlyArray<IArchiveStatisticsSection> = listArchiveStatisticsSections(statistics);

  // Falls back rather than rendering nothing: a subject can change under an open dialog, and the section that was
  // selected may be one the new one cannot answer.
  const selected: Optional<IArchiveStatisticsSection> =
    sections.find((it: IArchiveStatisticsSection) => it.id === selectedId) ?? sections[0];

  const view = useMemo(() => ({ measure, onMeasureChange: setMeasure }), [measure]);

  // Asked for on opening rather than when the subject is: most sessions never open this, and the answer costs a walk
  // of the whole listing. The service keeps it, so reopening is free.
  useEffect(() => {
    if (isOpen) {
      void archivesService.loadStatistics();
    }
  }, [archivesService, isOpen]);

  return (
    <Dialog
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-labelledby={titleId}
      fullWidth
      maxWidth={"md"}
      open={isOpen}
      onClose={onClose}
    >
      <DialogHeader
        title={"Statistics"}
        titleId={titleId}
        icon={<QueryStatsIcon fontSize={"small"} />}
        closeLabel={"Close statistics"}
        onClose={onClose}
      />

      <DialogContent className={"flex h-115 max-h-[64vh] p-0"}>
        <List
          className={"w-37 shrink-0 overflow-y-auto border-r border-divider"}
          dense={true}
          disablePadding={true}
          sx={getWellFillSx}
        >
          {sections.map((it: IArchiveStatisticsSection) => (
            <ListItemButton key={it.id} selected={selected?.id === it.id} onClick={() => setSelectedId(it.id)}>
              <ListItemText primary={it.label} />
            </ListItemButton>
          ))}
        </List>

        <div className={"min-w-0 grow overflow-y-auto px-dialog py-4"}>
          {archivesService.statistics.error ? (
            <Alert severity={"error"}>
              {`Could not describe this archive: ${archivesService.statistics.error.message}`}
            </Alert>
          ) : null}

          {archivesService.statistics.isLoading ? (
            <div className={"flex justify-center py-8"}>
              <CircularProgress size={24} />
            </div>
          ) : null}

          {statistics && selected ? selected.render(statistics, view) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}
