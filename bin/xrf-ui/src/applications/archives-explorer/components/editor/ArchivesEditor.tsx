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

  const project: Nullable<ArchiveProject> = archivesService.project.value;

  const archiveCount: number = project?.archives.length ?? 0;
  const fileCount: number = archivesService.files.length;
  const totalSize: number = project?.sizeReal ?? 0;
  const projectRoot: string = project?.root ?? "";

  // Extraction writes to disk outside the archive. Walking away mid-write left it running against a
  // screen nobody could see, and the only signal it was happening was one button in the content area.
  const isExtracting: boolean = archivesService.operation.isLoading;
  const isBusy: boolean = isClosing || isExtracting;

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

  useEditorBusy(isBusy);

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
        closeError ? (
          <Alert severity={"error"} onClose={() => setCloseError(null)}>
            Could not close archives: {closeError}
          </Alert>
        ) : null
      }
    >
      <ArchivesFileContent />
    </EditorLayout>
  );
}
