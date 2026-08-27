import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { IconButton, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { EquipmentService } from "@/core/equipment-icons";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Toolbar command that rebuilds the open sprite from its unpacked icons.
 */
export function EquipmentRepackAction(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const equipmentService: EquipmentService = useInjection(EquipmentService);

  const [isConfirmOpen, setConfirmOpen] = useState<boolean>(false);

  const isLoading: boolean = equipmentService.spriteImage.isLoading;
  const repackSourcePath: Nullable<string> = equipmentService.repackSourcePath;
  const spritePath: Nullable<string> = equipmentService.spriteImage.value?.path ?? null;

  const onRepack = useCallback(async () => {
    setConfirmOpen(false);

    try {
      await equipmentService.repackAndOpenProject();
    } catch (error) {
      // Already published as the sprite failure, which the editor renders. Logged here for the stack.
      log.error("Failed to repack and reopen DDS:", error);
    }
  }, [log, equipmentService]);

  const onOpenConfirmation = useCallback(() => setConfirmOpen(true), []);

  const onDeclineConfirmation = useCallback(() => setConfirmOpen(false), []);

  return (
    <>
      <Tooltip
        describeChild
        title={repackSourcePath ? "Rebuild the sprite from its unpacked icons" : "No unpacked icons beside the sprite"}
      >
        <span>
          <IconButton
            aria-label={"Repack sprite"}
            color={"inherit"}
            size={"small"}
            disabled={isLoading || !repackSourcePath}
            onClick={onOpenConfirmation}
          >
            <Inventory2Icon fontSize={"small"} />
          </IconButton>
        </span>
      </Tooltip>

      <ConfirmDialog
        isOpen={isConfirmOpen}
        isDestructive={true}
        title={"Repack sprite?"}
        confirmLabel={"Repack"}
        description={
          <>
            The sprite is rebuilt from the icons in
            <Typography component={"div"} variant={"caption"} className={"monospace"} sx={{ paddingY: 0.5 }}>
              {repackSourcePath}
            </Typography>
            overwriting
            <Typography component={"div"} variant={"caption"} className={"monospace"} sx={{ paddingY: 0.5 }}>
              {spritePath}
            </Typography>
            This cannot be undone.
          </>
        }
        onConfirm={onRepack}
        onClose={onDeclineConfirmation}
      />
    </>
  );
}
