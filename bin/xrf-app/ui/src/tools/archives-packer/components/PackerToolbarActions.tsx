import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorTextAction } from "@xrf/ui/editor/text-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ArchiveIcon } from "@xrf/ui/icons/archive";
import { FileOpenIcon } from "@xrf/ui/icons/file-open";
import { SaveAltIcon } from "@xrf/ui/icons/save-alt";
import { Stack } from "@xrf/ui/material/stack";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

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
      className={cn("mr-1 items-center", className)}
      direction={"row"}
      spacing={0.5}
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
