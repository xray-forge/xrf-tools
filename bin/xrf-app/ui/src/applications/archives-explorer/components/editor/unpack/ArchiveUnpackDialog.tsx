import { useInjection } from "@wirestate/react";
import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";
import { DialogHeader } from "@xrf/ui/dialog/header";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { UnarchiveIcon } from "@xrf/ui/icons";
import { Button, Dialog, DialogActions, DialogContent, Typography } from "@xrf/ui/material";
import { ReactElement, useCallback, useId } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { getSubjectRoot, getSubjectSize } from "@/core/archive/lib";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

interface IArchiveUnpackDialogProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Asks where to write every file the open subject holds, then starts that run and closes.
 */
export function ArchiveUnpackDialog({
  "data-testid": dataTestId = "archive-unpack-dialog",
  id,
  className,
  isOpen,
  onClose,
}: IArchiveUnpackDialogProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const titleId: string = useId();

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const isWriting: boolean = archivesService.isWriting;

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_EXPLORER,
    id: "unpack-destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isWriting,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_EXPLORER),
  });

  const onUnpack = useCallback(async (): Promise<void> => {
    if (!destination.value) {
      return;
    }

    // No enclosing form to tell the field it was used, so the run itself records it.
    destination.commit();
    onClose();

    // How it ends is published on the extraction and told by the run's notice.
    await archivesService.extractDirectory("", destination.value);
  }, [archivesService, destination, onClose]);

  return (
    <Dialog
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-labelledby={titleId}
      fullWidth
      maxWidth={"sm"}
      open={isOpen}
      onClose={onClose}
    >
      <DialogHeader
        title={"Unpack"}
        titleId={titleId}
        icon={<UnarchiveIcon fontSize={"small"} />}
        closeLabel={"Close unpack"}
        onClose={onClose}
      />

      <DialogContent className={"flex flex-col gap-4"}>
        <div>
          <Typography className={"font-monospace wrap-anywhere"} variant={"body2"}>
            {getSubjectRoot(subject)}
          </Typography>

          <Typography className={"text-text-secondary"} variant={"caption"}>
            {formatCount(archivesService.entries.length)} files · {formatBytes(getSubjectSize(subject))}
          </Typography>
        </div>

        <PathFormRow
          isDisabled={isWriting}
          label={"Output"}
          description={"Directory the whole tree is unpacked into, with its folder layout"}
          field={destination}
        />
      </DialogContent>

      <DialogActions>
        <Button size={"small"} onClick={onClose}>
          Cancel
        </Button>

        <Button
          size={"small"}
          variant={"contained"}
          disabled={isWriting || !destination.isValid || !archivesService.entries.length}
          startIcon={<UnarchiveIcon fontSize={"small"} />}
          onClick={() => void onUnpack()}
        >
          Unpack
        </Button>
      </DialogActions>
    </Dialog>
  );
}
