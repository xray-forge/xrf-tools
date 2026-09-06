import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { splitLogicalPath } from "@/core/ui/tree/path-tree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";
import { getFileExtension } from "@/lib/path/extension";
import { Nullable } from "@/lib/types/general";

export interface IArchiveFileExtractActionProps extends BaseComponentProps {
  descriptor: ArchiveFileDescriptor;
}

/**
 * Writes the selected archived file out to disk.
 */
export function ArchiveFileExtractAction({
  "data-testid": dataTestId,
  id,
  className,
  descriptor,
}: IArchiveFileExtractActionProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isExtracting: boolean = archivesService.operation.isLoading;

  const onExtract = useCallback(async () => {
    // The archived name is a full logical path; only its leaf is a file name.
    const suggested: string = splitLogicalPath(descriptor.name).name;

    const extension: string = getFileExtension(descriptor.name);
    const destination: Nullable<string> = await dialog.save({
      title: "Extract file",
      defaultPath: suggested,
      filters: extension ? [{ name: `${extension.toUpperCase()} file`, extensions: [extension] }] : undefined,
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
    <EditorIconAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Extract file"}
      description={"Extract this file to disk"}
      icon={<SaveAltIcon />}
      isDisabled={isExtracting}
      onClick={onExtract}
    />
  );
}
