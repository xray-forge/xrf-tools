import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { IconButton, Tooltip } from "@mui/material";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { splitLogicalPath } from "@/core/ui/tree/path-tree";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export interface IArchiveFileExtractActionProps {
  descriptor: ArchiveFileDescriptor;
}

/**
 * Writes the selected archived file out to disk.
 */
export function ArchiveFileExtractAction({ descriptor }: IArchiveFileExtractActionProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isExtracting: boolean = archivesService.operation.isLoading;

  const onExtract = useCallback(async () => {
    // The archived name is a full logical path; only its leaf is a file name.
    const suggested: string = splitLogicalPath(descriptor.name).name;

    const destination: Nullable<string> = await dialog.save({
      title: "Extract file",
      defaultPath: suggested,
      filters: descriptor.extension
        ? [{ name: `${descriptor.extension.toUpperCase()} file`, extensions: [descriptor.extension] }]
        : undefined,
    });

    if (!destination) {
      return;
    }

    try {
      await archivesService.extractFile(descriptor, destination);
    } catch (error) {
      // Published on the service as the extraction failure, which the header reports. Logged for the stack.
      log.error("Failed to extract archive file:", error);
    }
  }, [archivesService, descriptor, log]);

  return (
    <Tooltip describeChild title={"Extract this file to disk"}>
      <span>
        <IconButton
          aria-label={"Extract file"}
          color={"inherit"}
          disabled={isExtracting}
          size={"small"}
          onClick={onExtract}
        >
          <SaveAltIcon fontSize={"small"} />
        </IconButton>
      </span>
    </Tooltip>
  );
}
