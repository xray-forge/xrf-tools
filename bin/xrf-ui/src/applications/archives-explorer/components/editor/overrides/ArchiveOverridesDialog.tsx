import { default as ContentCopyIcon } from "@mui/icons-material/ContentCopy";
import { Alert, CircularProgress, Dialog, DialogContent, TextField } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback, useEffect, useId, useMemo, useState } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveWorldEntry } from "@/core/ipc/types/xrf-app";
import { ArchiveSourceUsage, ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { EStatMeasure } from "@/core/ui/stats/stat-measure";
import { IStatBreakdownRow, StatBreakdownTable } from "@/core/ui/stats/StatBreakdownTable";
import { StatMeasureToggle } from "@/core/ui/stats/StatMeasureToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

import { filterOverrides, flattenOverrides, TArchiveOverrideRow } from "./archive-override-rows";
import { ArchiveOverrideList } from "./ArchiveOverrideList";
import { ArchiveUnreachableSection } from "./ArchiveUnreachableSection";

export interface IArchiveOverridesDialogProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
  /** Opens the file a row names, which is what closes this dialog. */
  onOpenFile: (name: string) => void;
}

/**
 * Which engine paths this subject holds more than once, and what each contest buries.
 */
export function ArchiveOverridesDialog({
  "data-testid": dataTestId = "archive-overrides-dialog",
  id,
  className,
  isOpen,
  onClose,
  onOpenFile,
}: IArchiveOverridesDialogProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const titleId: string = useId();

  const [measure, setMeasure] = useState<EStatMeasure>(EStatMeasure.BYTES);
  const [filter, setFilter] = useState<string>("");
  const [source, setSource] = useState<Nullable<string>>(null);

  const overridden: Array<ArchiveWorldEntry> = archivesService.overridden;
  const statistics: Nullable<ArchiveStatistics> = archivesService.statistics.value;

  const isFolded: boolean = !archivesService.overrides.isLoading && !archivesService.overrides.error;

  const sources: Array<IStatBreakdownRow> = useMemo(
    () =>
      (statistics?.origins.sources ?? [])
        .filter((usage: ArchiveSourceUsage) => usage.hides.files > 0)
        .map((usage: ArchiveSourceUsage) => ({
          files: usage.hides.files,
          id: usage.source,
          label: usage.source,
          sizeReal: usage.hides.sizeReal,
        })),
    [statistics]
  );

  const entries: Array<ArchiveWorldEntry> = useMemo(
    () => filterOverrides(overridden, filter.trim().toLowerCase(), source),
    [overridden, filter, source]
  );

  const rows: Array<TArchiveOverrideRow> = useMemo(() => flattenOverrides(entries), [entries]);

  const hiddenCopies: number = useMemo(
    () => overridden.reduce((total: number, entry: ArchiveWorldEntry) => total + entry.shadowed.length, 0),
    [overridden]
  );

  const onSelectSource = useCallback((id: string) => setSource((current) => (current === id ? null : id)), []);

  const onOpen = useCallback(
    (name: string) => {
      onOpenFile(name);
      onClose();
    },
    [onClose, onOpenFile]
  );

  const onFilterChange = useCallback((event: ChangeEvent<HTMLInputElement>) => setFilter(event.target.value), []);

  // The aggregate is the backend's, so the two halves of this dialog cannot disagree about what a source hides.
  useEffect(() => {
    if (isOpen) {
      void archivesService.loadStatistics();
    }
  }, [archivesService, isOpen]);

  return (
    <Dialog
      data-testid={dataTestId}
      aria-labelledby={titleId}
      id={id}
      className={className}
      fullWidth
      maxWidth={"md"}
      open={isOpen}
      onClose={onClose}
    >
      <DialogHeader
        title={"Overrides"}
        titleId={titleId}
        icon={<ContentCopyIcon fontSize={"small"} />}
        closeLabel={"Close overrides"}
        onClose={onClose}
      />

      <DialogContent className={"flex h-115 max-h-dialog flex-col gap-4 px-dialog"}>
        {archivesService.overrides.error ? (
          <Alert severity={"error"}>
            {`Could not describe what this overrides: ${archivesService.overrides.error.message}`}
          </Alert>
        ) : null}

        {archivesService.overrides.isLoading || archivesService.statistics.isLoading ? (
          <div className={"flex justify-center py-8"}>
            <CircularProgress size={24} />
          </div>
        ) : null}

        {overridden.length === 0 && !archivesService.overrides.isLoading ? (
          <DetailSection
            data-testid={"archive-overrides-empty-section"}
            title={"Overridden paths"}
            description={
              "No engine path here is held more than once, so nothing is being replaced and nothing is buried. " +
              "Every file the tree lists is the only copy of itself."
            }
            fact={"0 paths"}
          />
        ) : null}

        {overridden.length > 0 ? (
          <>
            <DetailSection
              data-testid={"archive-overrides-sources-section"}
              className={"shrink-0"}
              title={"By source"}
              description={
                "A source claims an engine path a lower-priority one also holds, and the lower copy stays on disk " +
                "unread. Resolution says why the order is what it is; it counts mounts, while these are the volumes " +
                "and loose roots inside them, so the two lists are different lengths. Pick one to narrow the paths " +
                "below, and pick it again to clear."
              }
              fact={`${formatBytes(statistics?.origins.hidden.sizeReal ?? 0)} hidden`}
              action={<StatMeasureToggle measure={measure} onChange={setMeasure} />}
            >
              <div className={"max-h-40 overflow-y-auto"}>
                <StatBreakdownTable rows={sources} measure={measure} selectedId={source} onSelect={onSelectSource} />
              </div>
            </DetailSection>

            <DetailSection
              data-testid={"archive-overrides-paths-section"}
              className={"flex min-h-0 grow flex-col"}
              title={"Overridden paths"}
              description={
                "Each path with every copy claiming it, highest priority first. The top copy is the one the engine " +
                "loads; the rest stay on disk and are never read. This is how a mod replaces a file, not an error. " +
                `${overridden.length} path(s) are held more than once, by ${hiddenCopies} buried cop(ies).`
              }
              fact={`${entries.length} of ${overridden.length} path(s)`}
            >
              <TextField
                className={"mb-2 shrink-0"}
                size={"small"}
                fullWidth
                value={filter}
                placeholder={"Filter by engine path"}
                slotProps={{ htmlInput: { "aria-label": "Filter overridden paths" } }}
                onChange={onFilterChange}
              />

              <div className={"min-h-0 grow"}>
                <ArchiveOverrideList rows={rows} ariaLabel={"Overridden paths"} onOpen={onOpen} />
              </div>
            </DetailSection>
          </>
        ) : null}

        {isFolded ? (
          <ArchiveUnreachableSection className={"shrink-0"} collisions={archivesService.unreachable} />
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
