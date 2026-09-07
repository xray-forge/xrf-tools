import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Toolbar command that rebuilds the open sprite from its unpacked icons.
 */
export function EquipmentRepackAction(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);

  const [isConfirmOpen, setConfirmOpen] = useState<boolean>(false);

  const isLoading: boolean = spriteEquipmentService.spriteImage.isLoading;
  const repackSourcePath: Nullable<string> = spriteEquipmentService.repackSourcePath;
  const spritePath: Nullable<string> = spriteEquipmentService.spriteImage.value?.path ?? null;

  const onRepack = useCallback(async () => {
    setConfirmOpen(false);

    try {
      await spriteEquipmentService.repackAndOpenProject();
    } catch (error) {
      // Already published as the sprite failure, which the editor renders. Logged here for the stack.
      log.error("Failed to repack and reopen DDS:", error);
    }
  }, [log, spriteEquipmentService]);

  const onOpenConfirmation = useCallback(() => setConfirmOpen(true), []);

  const onDeclineConfirmation = useCallback(() => setConfirmOpen(false), []);

  return (
    <>
      <EditorIconAction
        label={"Repack sprite"}
        description={
          repackSourcePath ? "Rebuild the sprite from its unpacked icons" : "No unpacked icons beside the sprite"
        }
        icon={<Inventory2Icon />}
        isDisabled={isLoading || !repackSourcePath}
        onClick={onOpenConfirmation}
      />

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
