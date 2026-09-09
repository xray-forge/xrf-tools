import { default as CompareArrowsIcon } from "@mui/icons-material/CompareArrows";
import { default as DifferenceIcon } from "@mui/icons-material/Difference";
import { default as FileOpenIcon } from "@mui/icons-material/FileOpen";
import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { Button, Stack, Tooltip } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";

/** Sized to the caption row rather than to a page button, matching the packer's own action. */
const ACTION_STYLE = {
  height: 24,
  minWidth: 0,
  px: 1,
  fontSize: "0.75rem",
  lineHeight: 1,
  "& .MuiButton-startIcon": { mr: 0.5 },
} as const;

interface IPatcherToolbarActionsProps {
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
  isBusy,
  isRunDisabled,
  onImport,
  onExport,
  onCompare,
  onPatch,
}: IPatcherToolbarActionsProps): ReactElement {
  const disabledReason: string = "Choose a game and an output first";

  function action(
    label: string,
    title: string,
    icon: ReactNode,
    isPrimary: boolean,
    onClick: () => void
  ): ReactElement {
    return (
      <Tooltip describeChild title={isRunDisabled ? disabledReason : title}>
        <span>
          <Button
            size={"small"}
            variant={isPrimary ? "contained" : "outlined"}
            disabled={isRunDisabled}
            startIcon={icon}
            sx={ACTION_STYLE}
            onClick={onClick}
          >
            {label}
          </Button>
        </span>
      </Tooltip>
    );
  }

  return (
    <Stack direction={"row"} spacing={0.5} sx={{ alignItems: "center", mr: 0.5 }}>
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

      {action("Compare", "Report what differs without writing", <CompareArrowsIcon />, false, onCompare)}
      {action("Patch", "Write the patch volumes", <DifferenceIcon />, true, onPatch)}
    </Stack>
  );
}
