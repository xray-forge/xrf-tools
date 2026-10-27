import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorTextAction } from "@xrf/ui/editor/text-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CompareArrowsIcon } from "@xrf/ui/icons/compare-arrows";
import { DifferenceIcon } from "@xrf/ui/icons/difference";
import { FileOpenIcon } from "@xrf/ui/icons/file-open";
import { SaveAltIcon } from "@xrf/ui/icons/save-alt";
import { Stack } from "@xrf/ui/material/stack";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

interface IPatcherToolbarActionsProps extends BaseComponentProps {
  isBusy: boolean;
  isRunDisabled: boolean;
  onImport: () => void;
  onExport: () => void;
  onCompare: () => void;
  onPatch: () => void;
}

/**
 * Toolbar actions for the patcher.
 */
export function PatcherToolbarActions({
  "data-testid": dataTestId = "patcher-toolbar-actions",
  id,
  className,
  isBusy,
  isRunDisabled,
  onImport,
  onExport,
  onCompare,
  onPatch,
}: IPatcherToolbarActionsProps): ReactElement {
  const disabledReason: string = "Choose a game and an output first";

  return (
    <Stack
      data-testid={dataTestId}
      id={id}
      className={cn("mr-1 items-center", className)}
      direction={"row"}
      spacing={0.5}
    >
      <EditorIconAction
        label={"Import patching configuration"}
        description={"Import a patching configuration"}
        icon={<FileOpenIcon />}
        isDisabled={isBusy}
        onClick={onImport}
      />

      <EditorIconAction
        label={"Export patching configuration"}
        description={"Export this scope and header as a patching configuration"}
        icon={<SaveAltIcon />}
        isDisabled={isBusy}
        onClick={onExport}
      />

      <EditorTextAction
        label={"Compare"}
        description={isRunDisabled ? disabledReason : "Report what differs without writing"}
        icon={<CompareArrowsIcon />}
        isDisabled={isRunDisabled}
        onClick={onCompare}
      />

      <EditorTextAction
        label={"Patch"}
        description={isRunDisabled ? disabledReason : "Write the patch volumes"}
        icon={<DifferenceIcon />}
        isDisabled={isRunDisabled}
        variant={"contained"}
        onClick={onPatch}
      />
    </Stack>
  );
}
