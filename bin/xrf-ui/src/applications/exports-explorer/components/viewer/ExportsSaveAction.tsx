import { default as SaveAltIcon } from "@mui/icons-material/SaveAlt";
import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  DEFAULT_MANIFEST_NAME,
  MANIFEST_FILTERS,
  withExternManifestExtension,
} from "@/applications/exports-explorer/lib/extern-manifest";
import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IExportsSaveActionProps extends BaseComponentProps {
  isDisabled?: boolean;
}

/**
 * Writes the open project's externs out as one of the manifests `xrf-cli externs export` publishes.
 */
export function ExportsSaveAction({
  "data-testid": dataTestId = "exports-save-action",
  id,
  className,
  isDisabled = false,
}: IExportsSaveActionProps): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  const onSave = useCallback(async (): Promise<void> => {
    const selected: Nullable<string> = await dialog.save({
      title: "Save exports manifest",
      defaultPath: DEFAULT_MANIFEST_NAME,
      filters: MANIFEST_FILTERS,
    });

    if (!selected) {
      return;
    }

    // The extension is what names the format, and a save dialog answers with whatever was typed.
    await exportsService.exportManifest(withExternManifestExtension(selected));
  }, [exportsService]);

  return (
    <EditorIconAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Save exports"}
      description={"Save exports as a JSON, XML, or HTML manifest"}
      icon={<SaveAltIcon />}
      isDisabled={isDisabled}
      onClick={() => void onSave()}
    />
  );
}
