import { BaseComponentProps } from "@xrf/ui/element-types";
import { Stack } from "@xrf/ui/layout/stack";
import { ReactElement, ReactNode } from "react";

import { ArchiveVolumeOptionsFields } from "@/core/archive/components/ArchiveVolumeOptionsFields";
import { IArchiveConfigEditor, IArchiveEditedConfig } from "@/core/archive/services/archive-config-editor.service";

interface IArchiveConfigOptionsSectionProps extends BaseComponentProps {
  editor: IArchiveConfigEditor;
  /** The editor's open configuration. */
  config: IArchiveEditedConfig;
  /** Prefix of the fields' control ids. */
  fieldId: string;
  isDisabled?: boolean;
  /** The tool's own options, below the volume options every archive configuration has. */
  children?: ReactNode;
}

/**
 * How the volumes are written, as opposed to what goes into them. The volume ceiling is the editor's per-run one,
 * bounded by the ceiling the backend reports, so the form carries no copy of the format's limit.
 */
export function ArchiveConfigOptionsSection({
  "data-testid": dataTestId = "archive-config-options-section",
  id,
  className,
  editor,
  config,
  fieldId,
  isDisabled,
  children,
}: IArchiveConfigOptionsSectionProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <ArchiveVolumeOptionsFields
        id={fieldId}
        config={config}
        maxVolumeSizeMegabytes={editor.maxVolumeSizeMegabytes}
        volumeSize={editor.volumeSize}
        volumeSizeError={editor.volumeSizeError}
        isDisabled={isDisabled}
        onVolumeSizeChange={editor.setVolumeSize}
        onChange={editor.patchConfig}
      />

      {children}
    </Stack>
  );
}
