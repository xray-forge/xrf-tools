import { CommandBus } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveSubject, EArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable, Optional } from "@xrf/types";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { EditorToolbarLocation, IEditorLocation } from "@xrf/ui/editor/toolbar";
import { BarChartIcon, FileCopyIcon, LayersIcon, UnarchiveIcon } from "@xrf/ui/icons";
import { ReactElement, useCallback, useState } from "react";

import { ArchivePreviewService } from "@/applications/archives-explorer/services/archive-preview";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  getEntryEngineIdentity,
  getSubjectRoot,
  getSubjectSize,
  getSubjectSourceCount,
  IArchiveEntry,
} from "@/core/archive/lib";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { IPanelSetActiveCommand, PANEL_SET_ACTIVE_COMMAND } from "@/core/shell/panel/panel-messages";

import { ARCHIVE_EXPLORER_PANELS, EArchivePanelId } from "./archive-panels";
import { ArchiveOverridesDialog } from "./overrides";
import { ArchivesFilePreview } from "./preview";
import { ArchiveResolutionDialog } from "./resolution";
import { ArchiveStatisticsDialog } from "./statistics";
import { ArchiveUnpackDialog } from "./unpack";

export function ArchivesEditor(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);
  const commandBus: CommandBus = useInjection(CommandBus);

  const [isStatisticsOpen, setStatisticsOpen] = useState<boolean>(false);
  const [isResolutionOpen, setResolutionOpen] = useState<boolean>(false);
  const [isOverridesOpen, setOverridesOpen] = useState<boolean>(false);
  const [isUnpackOpen, setUnpackOpen] = useState<boolean>(false);

  // The run rather than the service's own flag: an extraction survives the window being reloaded, so returning here
  // finds it again instead of showing an idle tree over files it is still writing.
  const job: Nullable<IJobState> = archivesService.extraction.job;

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const root: string = getSubjectRoot(subject);
  const overriddenCount: number = archivesService.overridden.length;

  const isWorld: boolean = subject?.kind === EArchiveSubject.WORLD;
  const isBusy: boolean = archivesService.subject.isLoading || archivesService.isWriting;

  const location: Nullable<IEditorLocation> = root ? { path: root } : null;

  const onCancelExtraction = useCallback(() => archivesService.extraction.cancel(), [archivesService]);

  const onClose = useCallback((): void => void archivesService.closeSubject(), [archivesService]);

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

      void previewService.selectFile(entry);
    },
    [archivesService, commandBus, previewService]
  );

  useEditorPanels(() => ARCHIVE_EXPLORER_PANELS, []);

  useEditorBusy(isBusy || Boolean(job));

  useEditorStatus([
    `${formatCount(getSubjectSourceCount(subject))} ${isWorld ? "sources" : "archives"}`,
    `${formatCount(archivesService.entries.length)} files`,
    formatBytes(getSubjectSize(subject)),
    ...(overriddenCount ? [`${formatCount(overriddenCount)} overridden`] : []),
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

              <EditorIconAction
                aria-haspopup={"dialog"}
                aria-expanded={isUnpackOpen}
                label={"Unpack"}
                description={"Write every file in the open tree to a directory"}
                icon={<UnarchiveIcon />}
                isDisabled={archivesService.isWriting || !archivesService.entries.length}
                onClick={() => setUnpackOpen(true)}
              />
            </>
          }
          onBack={onClose}
        />
      }
      banner={
        job ? (
          <div className={"px-4 py-2"}>
            <JobProgressView job={job} onCancel={onCancelExtraction} />
          </div>
        ) : null
      }
    >
      <ArchivesFilePreview onUnpack={() => setUnpackOpen(true)} />

      <ArchiveResolutionDialog isOpen={isResolutionOpen} onClose={() => setResolutionOpen(false)} />

      <ArchiveStatisticsDialog isOpen={isStatisticsOpen} onClose={() => setStatisticsOpen(false)} />

      <ArchiveOverridesDialog
        isOpen={isOverridesOpen}
        onClose={() => setOverridesOpen(false)}
        onOpenFile={onOpenOverride}
      />

      <ArchiveUnpackDialog isOpen={isUnpackOpen} onClose={() => setUnpackOpen(false)} />
    </EditorLayout>
  );
}
