import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { Alert, Box, Button, Typography } from "@mui/material";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { ARCHIVE_EDITOR_MONOSPACE_FONT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { isUnderArchiveDirectory, TArchiveOperation } from "@/core/archive";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { Loadable } from "@/lib/loadable";
import { Logger, useLogger } from "@/lib/logging";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

export interface IArchiveDirectoryContentProps {
  path: string;
}

/**
 * What the content area shows when a directory is selected rather than a file.
 *
 * A directory has nothing to preview, so it reports what extracting it would cost - how many files and how
 * much data - and offers the command. The counts are computed here rather than taken from the tree
 * because the tree only knows its own shape, while the totals people care about are recursive.
 */
export function ArchiveDirectoryContent({ path }: IArchiveDirectoryContentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const files: Array<ArchiveFileDescriptor> = archivesService.files;
  const operation: Loadable<Nullable<TArchiveOperation>> = archivesService.operation;
  // A file extraction started elsewhere must not be reported here as if this directory had been written.
  const extracted: Nullable<ArchiveExtractDirectoryResult> =
    operation.value?.kind === "extract-directory" ? operation.value.result : null;

  const summary = useMemo(() => {
    let count: number = 0;
    let size: number = 0;

    // Same rule the backend extracts by, so the promised count is the delivered one.
    for (const descriptor of files) {
      if (isUnderArchiveDirectory(descriptor, path)) {
        count += 1;
        size += descriptor.sizeReal;
      }
    }

    return { count, size };
  }, [files, path]);

  const onExtract = useCallback(async () => {
    const destination: Nullable<string> = (await dialog.open({
      title: path ? `Extract ${path}` : "Extract archive",
      directory: true,
    })) as Nullable<string>;

    if (!destination) {
      return;
    }

    try {
      await archivesService.extractArchiveDirectory(path, destination);
    } catch (error: unknown) {
      // Published on the service, which the alert below renders. Logged here for the stack.
      log.error("Failed to extract archive directory:", error);
    }
  }, [archivesService, log, path]);

  return (
    <CenteredColumn sx={{ padding: 3, gap: 1 }}>
      <FolderOpenIcon sx={{ color: "text.secondary" }} />

      <Typography variant={"subtitle1"} sx={{ fontFamily: ARCHIVE_EDITOR_MONOSPACE_FONT, overflowWrap: "anywhere" }}>
        {path || "Archive root"}
      </Typography>

      <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
        {summary.count} files · {formatBytes(summary.size)}
      </Typography>

      <Button
        variant={"contained"}
        size={"small"}
        disabled={operation.isLoading || !summary.count}
        startIcon={<SaveAltIcon fontSize={"small"} />}
        sx={{ marginTop: 1 }}
        onClick={onExtract}
      >
        {operation.isLoading ? "Extracting..." : "Extract directory"}
      </Button>

      {operation.error ? (
        <Box sx={{ marginTop: 2, maxWidth: 480 }}>
          <Alert severity={"error"} variant={"outlined"} onClose={archivesService.clearOperation}>
            <Typography variant={"caption"} sx={{ wordBreak: "break-word" }}>
              {String(operation.error)}
            </Typography>
          </Alert>
        </Box>
      ) : null}

      {extracted ? (
        <Box sx={{ marginTop: 2, maxWidth: 480 }}>
          <Alert severity={"success"} variant={"outlined"} onClose={archivesService.clearOperation}>
            <Typography variant={"caption"} sx={{ wordBreak: "break-word" }}>
              {`Extracted ${extracted.extractedCount} files to ${extracted.destination}`}
            </Typography>
          </Alert>
        </Box>
      ) : null}
    </CenteredColumn>
  );
}
