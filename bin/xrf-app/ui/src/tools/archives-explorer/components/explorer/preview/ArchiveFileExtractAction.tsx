import { useInjection } from "@wirestate/react";
import { getFileExtension, splitLogicalPath } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { SaveAltIcon } from "@xrf/ui/icons/save-alt";
import { ReactElement, useCallback } from "react";

import { IArchiveEntry } from "@/core/archive/lib/entry";
import { askToSave } from "@/core/path/native-dialog";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

export interface IArchiveFileExtractActionProps extends BaseComponentProps {
  entry: IArchiveEntry;
}

/**
 * Writes the selected file out to disk, whichever tree the explorer has open.
 */
export function ArchiveFileExtractAction({
  "data-testid": dataTestId,
  id,
  className,
  entry,
}: IArchiveFileExtractActionProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isWriting: boolean = archivesService.isWriting;

  const onExtract = useCallback(async () => {
    // The engine name is a full logical path; only its leaf is a file name.
    const suggested: string = splitLogicalPath(entry.name).name;

    const extension: string = getFileExtension(entry.name);
    const destination: Nullable<string> = await askToSave({
      title: "Extract file",
      start: suggested,
      filters: extension ? [{ name: `${extension.toUpperCase()} file`, extensions: [extension] }] : null,
    });

    if (destination) {
      // How it ends is told as a notice.
      await archivesService.extractFile(entry, destination);
    }
  }, [archivesService, entry]);

  return (
    <EditorIconAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Extract file"}
      description={"Extract this file to disk"}
      icon={<SaveAltIcon />}
      isDisabled={isWriting}
      onClick={onExtract}
    />
  );
}
