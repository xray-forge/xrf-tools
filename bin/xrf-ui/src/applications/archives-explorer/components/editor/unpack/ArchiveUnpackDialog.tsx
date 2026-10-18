import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";
import { Button, Dialog, DialogActions, DialogContent, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatBytes, formatCount } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useId } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { getSubjectRoot, getSubjectSize } from "@/core/archive/lib";
import { ArchiveSubject } from "@/core/ipc/types/xrf-app";
import { EApplicationId } from "@/core/routing/application";
import { resolveOutputPath } from "@/core/settings/lib/output-path";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

interface IArchiveUnpackDialogProps extends BaseComponentProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Asks where to write every file the open subject holds, then starts that run.
 *
 * Closes as soon as the run starts: progress and cancelling belong to the editor's banner, which is also where a run
 * started before a reload is found again.
 */
export function ArchiveUnpackDialog({
  "data-testid": dataTestId = "archive-unpack-dialog",
  id,
  className,
  isOpen,
  onClose,
}: IArchiveUnpackDialogProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const archivesService: ArchivesService = useInjection(ArchivesService);
  const titleId: string = useId();

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const isExtracting: boolean = archivesService.operation.isLoading;

  const destination: IPathField = usePathField({
    application: EApplicationId.ARCHIVES_EXPLORER,
    id: "unpack-destination",
    title: "Select output directory",
    isDirectory: true,
    isSave: true,
    isDisabled: isExtracting,
    seed: () => resolveOutputPath(EApplicationId.ARCHIVES_EXPLORER),
  });

  const onUnpack = useCallback(async (): Promise<void> => {
    if (!destination.value) {
      return;
    }

    // No enclosing form to tell the field it was used, so the run itself records it.
    destination.commit();
    onClose();

    try {
      await archivesService.extractArchiveDirectory("", destination.value);
    } catch (error: unknown) {
      // Published on the service and reported by the run's notice. Logged here for the stack.
      log.error("Failed to unpack archive tree:", error);
    }
  }, [archivesService, destination, log, onClose]);

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
          isDisabled={isExtracting}
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
          disabled={isExtracting || !destination.isValid || !archivesService.entries.length}
          startIcon={<UnarchiveIcon fontSize={"small"} />}
          onClick={() => void onUnpack()}
        >
          Unpack
        </Button>
      </DialogActions>
    </Dialog>
  );
}
