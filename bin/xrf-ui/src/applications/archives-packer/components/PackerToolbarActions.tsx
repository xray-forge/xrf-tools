import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { default as FileOpenIcon } from "@mui/icons-material/FileOpen";
import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { Stack } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorTextAction } from "@/core/shell/editor/EditorTextAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPackerToolbarActionsProps extends BaseComponentProps {
  isBusy: boolean;
  isPackDisabled: boolean;
  onImport: () => void;
  onExport: () => void;
  onPack: () => void;
}

/**
 * Toolbar actions for the packer.
 */
export function PackerToolbarActions({
  "data-testid": dataTestId = "packer-toolbar-actions",
  id,
  className,
  isBusy,
  isPackDisabled,
  onImport,
  onExport,
  onPack,
}: IPackerToolbarActionsProps): ReactElement {
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
        label={"Import packing configuration"}
        description={"Import a packing configuration"}
        icon={<FileOpenIcon />}
        isDisabled={isBusy}
        onClick={onImport}
      />

      <EditorIconAction
        label={"Export packing configuration"}
        description={"Export these rules as a packing configuration"}
        icon={<SaveAltIcon />}
        isDisabled={isBusy}
        onClick={onExport}
      />

      <EditorTextAction
        label={"Pack"}
        description={isPackDisabled ? "Choose a source and an output first" : "Write the volumes"}
        icon={<ArchiveIcon />}
        isDisabled={isPackDisabled}
        variant={"contained"}
        onClick={onPack}
      />
    </Stack>
  );
}
