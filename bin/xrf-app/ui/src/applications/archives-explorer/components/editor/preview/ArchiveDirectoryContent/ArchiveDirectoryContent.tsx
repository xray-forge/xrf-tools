import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";
import { Alert, Button, Typography } from "@mui/material";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveExtractDirectoryResult } from "@xrf/ipc/types/xrf-pack";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { resolveDirectoryDestination } from "@/applications/archives-explorer/lib/resolve-directory-destination";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IArchiveEntry, isUnderArchiveDirectory, TArchiveOperation } from "@/core/archive/lib";
import { cn } from "@/core/theme/class-name";
import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveDirectoryContentProps extends BaseComponentProps {
  path: string;
  /** Opens the unpack dialog, which the tree root offers instead of extracting into a bare picked folder. */
  onUnpack: () => void;
}

/**
 * What the content area shows when a directory is selected rather than a file.
 *
 * A directory has nothing to preview, so it reports what extracting it would cost - how many files and how
 * much data - and offers the command. The counts are computed here rather than taken from the tree
 * because the tree only knows its own shape, while the totals people care about are recursive.
 */
export function ArchiveDirectoryContent({
  "data-testid": dataTestId = "archive-directory-content",
  id,
  className,
  path,
  onUnpack,
}: IArchiveDirectoryContentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const files: Array<IArchiveEntry> = archivesService.entries;
  const operation: AsyncState<Nullable<TArchiveOperation>> = archivesService.operation;
  const isWriting: boolean = archivesService.isWriting;
  // A file extraction started elsewhere must not be reported here as if this directory had been written.
  const extracted: Nullable<ArchiveExtractDirectoryResult> =
    operation.value?.kind === "extract-directory" ? operation.value.result : null;

  const summary = useMemo(() => {
    let count: number = 0;
    let size: number = 0;

    // Same rule the backend extracts by, so the promised count is the delivered one.
    for (const entry of files) {
      if (isUnderArchiveDirectory(entry, path)) {
        count += 1;
        size += entry.sizeReal;
      }
    }

    return { count, size };
  }, [files, path]);

  const onExtract = useCallback(async () => {
    const picked: Nullable<string> = (await dialog.open({
      title: `Extract ${path} into`,
      directory: true,
    })) as Nullable<string>;

    if (!picked) {
      return;
    }

    try {
      await archivesService.extractArchiveDirectory(path, await resolveDirectoryDestination(picked, path));
    } catch (error: unknown) {
      // Published on the service, which the alert below renders. Logged here for the stack.
      log.error("Failed to extract archive directory:", error);
    }
  }, [archivesService, log, path]);

  return (
    <CenteredColumn data-testid={dataTestId} id={id} className={cn("gap-2 p-6", className)}>
      <FolderOpenIcon className={"text-text-secondary"} />

      <Typography className={"font-monospace wrap-anywhere"} variant={"subtitle1"}>
        {path || "Tree root"}
      </Typography>

      <Typography className={"text-text-secondary"} variant={"body2"}>
        {formatCount(summary.count)} files · {formatBytes(summary.size)}
      </Typography>

      {path ? (
        <Button
          variant={"contained"}
          size={"small"}
          disabled={isWriting || !summary.count}
          startIcon={<SaveAltIcon fontSize={"small"} />}
          onClick={onExtract}
        >
          {operation.isLoading ? "Extracting..." : "Extract directory"}
        </Button>
      ) : (
        <Button
          aria-haspopup={"dialog"}
          variant={"contained"}
          size={"small"}
          disabled={isWriting || !summary.count}
          startIcon={<UnarchiveIcon fontSize={"small"} />}
          onClick={onUnpack}
        >
          Unpack...
        </Button>
      )}

      {operation.error ? (
        <div className={"mt-4 max-w-120"}>
          <Alert severity={"error"} variant={"outlined"} onClose={archivesService.clearOperation}>
            <Typography className={"wrap-break-word"} variant={"caption"}>
              {operation.error.message}
            </Typography>
          </Alert>
        </div>
      ) : null}

      {extracted ? (
        <div className={"mt-4 max-w-120"}>
          <Alert severity={"success"} variant={"outlined"} onClose={archivesService.clearOperation}>
            <Typography className={"wrap-break-word"} variant={"caption"}>
              {`Extracted ${formatCount(extracted.extractedCount)} files to ${extracted.destination}`}
            </Typography>
          </Alert>
        </div>
      ) : null}
    </CenteredColumn>
  );
}
