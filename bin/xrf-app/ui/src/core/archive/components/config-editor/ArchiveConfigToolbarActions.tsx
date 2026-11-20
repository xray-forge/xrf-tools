import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FileOpenIcon } from "@xrf/ui/icons/file-open";
import { SaveAltIcon } from "@xrf/ui/icons/save-alt";
import { Stack } from "@xrf/ui/layout/stack";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode } from "react";

import {
  IArchiveConfigFile,
  useArchiveConfigFile,
} from "@/core/archive/components/config-editor/use-archive-config-file";
import { IArchiveConfigEditor } from "@/core/archive/services/archive-config-editor.service";

interface IArchiveConfigToolbarActionsProps extends BaseComponentProps {
  editor: IArchiveConfigEditor;
  /** What the configuration is called, such as "packing configuration". */
  noun: string;
  /** What exporting writes, in the export action's tooltip. */
  exportDescription: string;
  isBusy: boolean;
  /** The runs the tool offers, after import and export. */
  children: ReactNode;
}

/**
 * Toolbar actions of an archive configuration editor: import, export, and the tool's own runs.
 */
export function ArchiveConfigToolbarActions({
  "data-testid": dataTestId = "archive-config-toolbar-actions",
  id,
  className,
  editor,
  noun,
  exportDescription,
  isBusy,
  children,
}: IArchiveConfigToolbarActionsProps): ReactElement {
  const file: IArchiveConfigFile = useArchiveConfigFile({ editor, noun });

  return (
    <Stack
      data-testid={dataTestId}
      id={id}
      className={cn("mr-1 items-center", className)}
      direction={"row"}
      spacing={0.5}
    >
      <EditorIconAction
        label={`Import ${noun}`}
        description={`Import a ${noun}`}
        icon={<FileOpenIcon />}
        isDisabled={isBusy}
        onClick={() => void file.askToImport()}
      />

      <EditorIconAction
        label={`Export ${noun}`}
        description={exportDescription}
        icon={<SaveAltIcon />}
        isDisabled={isBusy}
        onClick={() => void file.askToExport()}
      />

      {children}
    </Stack>
  );
}
