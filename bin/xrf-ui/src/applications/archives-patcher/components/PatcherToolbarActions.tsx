import { default as CompareArrowsIcon } from "@mui/icons-material/CompareArrows";
import { default as DifferenceIcon } from "@mui/icons-material/Difference";
import { default as FileOpenIcon } from "@mui/icons-material/FileOpen";
import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { Stack } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorTextAction } from "@/core/shell/editor/EditorTextAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      className={className}
      direction={"row"}
      spacing={0.5}
      sx={{ alignItems: "center", mr: 0.5 }}
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
