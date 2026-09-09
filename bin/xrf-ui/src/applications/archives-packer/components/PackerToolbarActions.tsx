import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { default as FileOpenIcon } from "@mui/icons-material/FileOpen";
import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import { Stack } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorTextAction } from "@/core/shell/editor/EditorTextAction";

interface IPackerToolbarActionsProps {
  isBusy: boolean;
  isPackDisabled: boolean;
  onImport: () => void;
  onExport: () => void;
  onPack: () => void;
}

/**
 * Toolbar actions for the packer.
 *
 * The caption row is sized for icon buttons, so the two configuration actions are icons and only the
 * primary verb keeps its label. A tooltip carries what each icon means, and the disabled ones are
 * wrapped so the tooltip still reaches them.
 */
export function PackerToolbarActions({
  isBusy,
  isPackDisabled,
  onImport,
  onExport,
  onPack,
}: IPackerToolbarActionsProps): ReactElement {
  return (
    <Stack direction={"row"} spacing={0.5} sx={{ alignItems: "center", mr: 0.5 }}>
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
