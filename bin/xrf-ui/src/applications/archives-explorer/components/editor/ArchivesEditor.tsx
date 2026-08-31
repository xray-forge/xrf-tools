import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Alert, Box, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { ARCHIVE_EDITOR_MONOSPACE_FONT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { createArchiveEditorPanels } from "@/applications/archives-explorer/components/editor/archive-panels";
import { ArchivesFileContent } from "@/applications/archives-explorer/components/editor/preview/ArchivesFileContent";
import { ArchivesMenu } from "@/applications/archives-explorer/components/editor/tree/ArchivesMenu";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/EditorBusyContext";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

export function ArchivesEditor(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const [isClosing, setClosing] = useState<boolean>(false);
  const [closeError, setCloseError] = useState<Nullable<string>>(null);
  const [isCollisionNoticeDismissed, setCollisionNoticeDismissed] = useState<boolean>(false);

  const project: Nullable<ArchiveProject> = archivesService.project.value;
  const collisions: Array<XrayPathCollision> = archivesService.collisions.value ?? [];

  const archiveCount: number = project?.archives.length ?? 0;
  const fileCount: number = archivesService.files.length;
  const totalSize: number = project?.sizeReal ?? 0;
  const projectRoot: string = project?.root ?? "";

  // The run rather than the service's own flag: an extraction survives the window being reloaded, so returning here
  // finds it again instead of showing an idle tree over files it is still writing.
  const job: Nullable<IJobState> = archivesService.job;

  // Extraction writes to disk outside the archive. Walking away mid-write left it running against a
  // screen nobody could see, and the only signal it was happening was one button in the content area.
  const isExtracting: boolean = archivesService.operation.isLoading;
  const isBusy: boolean = isClosing || isExtracting;
  const isCollisionNoticeShown: boolean = collisions.length > 0 && !isCollisionNoticeDismissed;

  const onCancelExtraction = useCallback(() => archivesService.cancelExtraction(), [archivesService]);

  const onClose = useCallback(async (): Promise<void> => {
    setClosing(true);
    setCloseError(null);

    try {
      await archivesService.closeProject();
    } catch (error: unknown) {
      setCloseError(error instanceof Error ? error.message : String(error));
    } finally {
      setClosing(false);
    }
  }, [archivesService]);

  useEditorPanels(
    () => [
      {
        icon: <FolderOpenIcon />,
        id: "archives",
        isOpenByDefault: true,
        label: "Archives",
        render: () => <ArchivesMenu />,
        side: "left",
      },
      ...createArchiveEditorPanels(archivesService),
    ],
    [archivesService]
  );

  useEditorBusy(isBusy || Boolean(job));

  useEditorStatus([`${archiveCount} archives`, `${fileCount} files`, formatBytes(totalSize)]);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={
            projectRoot ? (
              <Tooltip title={projectRoot}>
                <Box component={"span"} sx={{ fontFamily: ARCHIVE_EDITOR_MONOSPACE_FONT }}>
                  {projectRoot}
                </Box>
              </Tooltip>
            ) : null
          }
          onBack={() => void onClose()}
        />
      }
      banner={
        job || closeError || isCollisionNoticeShown ? (
          <>
            {job ? (
              <Box sx={{ paddingX: 2, paddingY: 1 }}>
                <JobProgressView job={job} onCancel={onCancelExtraction} />
              </Box>
            ) : null}

            {closeError ? (
              <Alert severity={"error"} onClose={() => setCloseError(null)}>
                Could not close archives: {closeError}
              </Alert>
            ) : null}

            {isCollisionNoticeShown ? (
              <Alert
                severity={"warning"}
                closeText={"Dismiss unreachable files notice"}
                onClose={() => setCollisionNoticeDismissed(true)}
              >
                {collisions.length} file(s) here cannot be reached - another entry claims their engine path. See the
                Unreachable files panel.
              </Alert>
            ) : null}
          </>
        ) : null
      }
    >
      <ArchivesFileContent />
    </EditorLayout>
  );
}
