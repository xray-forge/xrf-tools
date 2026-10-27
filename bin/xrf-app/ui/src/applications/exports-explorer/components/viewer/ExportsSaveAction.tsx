import * as dialog from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { SaveAltIcon } from "@xrf/ui/icons/save-alt";
import { ReactElement, useCallback } from "react";

import {
  DEFAULT_MANIFEST_NAME,
  MANIFEST_FILTERS,
  withExternManifestExtension,
} from "@/applications/exports-explorer/lib/extern-manifest";
import { ExportsService } from "@/applications/exports-explorer/services/exports.service";

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
