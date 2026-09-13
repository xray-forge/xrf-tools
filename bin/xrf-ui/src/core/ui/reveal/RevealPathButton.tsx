import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Button } from "@mui/material";
import { ReactElement, useCallback, useState } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IRevealPathButtonProps extends BaseComponentProps {
  /** Which application the failure notification is attributed to. */
  application: EApplicationId;
  /** Path to show; a missing one disables the button rather than hiding it. */
  path: Nullable<string>;
  label?: string;
  isDisabled?: boolean;
}

/**
 * Opens a produced path in the desktop's file manager.
 *
 * A command that wrote something is only half useful if finding what it wrote means retyping the path,
 * so every surface that reports an output offers the same way to reach it.
 */
export function RevealPathButton({
  "data-testid": dataTestId = "reveal-path-button",
  id,
  className,
  application,
  path,
  label = "Show in file manager",
  isDisabled,
}: IRevealPathButtonProps): ReactElement {
  const notify: TEmitNotification = useEmitNotification();

  const [isRevealing, setIsRevealing] = useState<boolean>(false);

  const onReveal = useCallback(async () => {
    if (!path) {
      return;
    }

    try {
      setIsRevealing(true);

      await systemCommands.revealPath(path);
    } catch (error: unknown) {
      // Reported rather than thrown: failing to show a directory says nothing about the command that
      // filled it, and the result beside this button is still the answer the user came for.
      notify({
        details: `${path}\n${String(error)}`,
        severity: ENotificationSeverity.ERROR,
        source: application,
        title: "Could not open the file manager",
      });
    } finally {
      setIsRevealing(false);
    }
  }, [application, notify, path]);

  return (
    <Button
      data-testid={dataTestId}
      id={id}
      className={className}
      size={"small"}
      variant={"text"}
      disabled={isDisabled || isRevealing || !path}
      startIcon={<FolderOpenIcon fontSize={"small"} />}
      onClick={() => void onReveal()}
    >
      {label}
    </Button>
  );
}
