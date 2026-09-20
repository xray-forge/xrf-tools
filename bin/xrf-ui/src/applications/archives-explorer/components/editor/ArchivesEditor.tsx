import { default as BarChartIcon } from "@mui/icons-material/BarChart";
import { default as FileCopyIcon } from "@mui/icons-material/FileCopy";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { Alert } from "@mui/material";
import { CommandBus } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  getEntryEngineIdentity,
  getSubjectRoot,
  getSubjectSize,
  getSubjectSourceCount,
  IArchiveEntry,
} from "@/core/archive/lib";
import { ArchiveSubject, EArchiveSubject } from "@/core/ipc/types/xrf-app";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation, IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { IPanelSetActiveCommand, PANEL_SET_ACTIVE_COMMAND } from "@/core/shell/panel/panel-messages";
import { formatBytes } from "@/lib/memory/format";
import { Nullable, Optional } from "@/lib/types/general";

import { ARCHIVE_EXPLORER_PANELS, EArchivePanelId } from "./archive-panels";
import { ArchiveOverridesDialog } from "./overrides";
import { ArchivesFilePreview } from "./preview";
import { ArchiveResolutionDialog } from "./resolution";
import { ArchiveStatisticsDialog } from "./statistics";

export function ArchivesEditor(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const commandBus: CommandBus = useInjection(CommandBus);

  const [isClosing, setClosing] = useState<boolean>(false);
  const [closeError, setCloseError] = useState<Nullable<string>>(null);
  const [isStatisticsOpen, setStatisticsOpen] = useState<boolean>(false);
  const [isResolutionOpen, setResolutionOpen] = useState<boolean>(false);
  const [isOverridesOpen, setOverridesOpen] = useState<boolean>(false);

  // The run rather than the service's own flag: an extraction survives the window being reloaded, so returning here
  // finds it again instead of showing an idle tree over files it is still writing.
  const job: Nullable<IJobState> = archivesService.job;

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const root: string = getSubjectRoot(subject);
  const overriddenCount: number = archivesService.overridden.length;

  const isWorld: boolean = subject?.kind === EArchiveSubject.WORLD;
  const isExtracting: boolean = archivesService.operation.isLoading;
  const isBusy: boolean = isClosing || isExtracting;

  const location: Nullable<IEditorLocation> = root ? { path: root } : null;

  const onCancelExtraction = useCallback(() => archivesService.cancelExtraction(), [archivesService]);

  const onClose = useCallback(async (): Promise<void> => {
    setClosing(true);
    setCloseError(null);

    try {
      await archivesService.closeSubject();
    } catch (error: unknown) {
      setCloseError(error instanceof Error ? error.message : String(error));
    } finally {
      setClosing(false);
    }
  }, [archivesService]);

  /**
   * Opens the file a contested path names, and raises the tree that shows where it sits.
   */
  const onOpenOverride = useCallback(
    (name: string): void => {
      const entry: Optional<IArchiveEntry> = archivesService.entries.find(
        (candidate: IArchiveEntry) => getEntryEngineIdentity(candidate) === name
      );

      if (!entry) {
        return;
      }

      commandBus.execute<void, IPanelSetActiveCommand>(
        PANEL_SET_ACTIVE_COMMAND,
        { panelId: EArchivePanelId.FILES, side: "left" },
        { optional: true }
      );

      void archivesService.selectArchiveFile(entry);
    },
    [archivesService, commandBus]
  );

  useEditorPanels(() => ARCHIVE_EXPLORER_PANELS, []);

  useEditorBusy(isBusy || Boolean(job));

  useEditorStatus([
    `${getSubjectSourceCount(subject)} ${isWorld ? "sources" : "archives"}`,
    `${archivesService.entries.length} files`,
    formatBytes(getSubjectSize(subject)),
    ...(overriddenCount ? [`${overriddenCount} overridden`] : []),
  ]);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={location ? <EditorToolbarLocation location={location} /> : null}
          actions={
            <>
              <EditorIconAction
                aria-haspopup={"dialog"}
                aria-expanded={isResolutionOpen}
                label={"Resolution"}
                description={"Which sources are searched, and in what order"}
                icon={<LayersIcon />}
                onClick={() => setResolutionOpen(true)}
              />

              <EditorIconAction
                aria-haspopup={"dialog"}
                aria-expanded={isOverridesOpen}
                label={"Overrides"}
                description={"Engine paths held more than once, and what each contest buries"}
                icon={<FileCopyIcon />}
                onClick={() => setOverridesOpen(true)}
              />

              <EditorIconAction
                aria-haspopup={"dialog"}
                aria-expanded={isStatisticsOpen}
                label={"Statistics"}
                description={"What this archive holds, broken down"}
                icon={<BarChartIcon />}
                onClick={() => setStatisticsOpen(true)}
              />
            </>
          }
          onBack={() => void onClose()}
        />
      }
      banner={
        job || closeError ? (
          <>
            {job ? (
              <div className={"px-4 py-2"}>
                <JobProgressView job={job} onCancel={onCancelExtraction} />
              </div>
            ) : null}

            {closeError ? (
              <Alert severity={"error"} onClose={() => setCloseError(null)}>
                Could not close archives: {closeError}
              </Alert>
            ) : null}
          </>
        ) : null
      }
    >
      <ArchivesFilePreview />

      <ArchiveResolutionDialog isOpen={isResolutionOpen} onClose={() => setResolutionOpen(false)} />

      <ArchiveStatisticsDialog isOpen={isStatisticsOpen} onClose={() => setStatisticsOpen(false)} />

      <ArchiveOverridesDialog
        isOpen={isOverridesOpen}
        onClose={() => setOverridesOpen(false)}
        onOpenFile={onOpenOverride}
      />
    </EditorLayout>
  );
}
