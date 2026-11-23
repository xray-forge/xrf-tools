import { ArchivePackHeader } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { Stack } from "@xrf/ui/layout/stack";
import { ReactElement } from "react";

import { ArchiveHeaderFields } from "@/core/archive/components/ArchiveHeaderFields";
import { DEFAULT_ENTRY_POINT, HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive/lib/header";
import { IArchiveConfigEditor, IArchiveEditedConfig } from "@/core/archive/services/archive-config-editor.service";

interface IArchiveConfigHeaderSectionProps extends BaseComponentProps {
  editor: IArchiveConfigEditor;
  /** The editor's open configuration. */
  config: IArchiveEditedConfig;
  /** Prefix of the fields' control ids. */
  fieldId: string;
  entryPointDescription: string;
  /** Why volumes without an entry point are a problem for this tool. */
  missingEntryPointWarning: string;
  /** What to know about an entry point other than the default, or nothing where any is as good. */
  customEntryPointNotice?: Nullable<string>;
  isDisabled?: boolean;
}

/**
 * The header written into the volumes, which is how the engine decides where their contents mount.
 */
export function ArchiveConfigHeaderSection({
  "data-testid": dataTestId = "archive-config-header-section",
  id,
  className,
  editor,
  config,
  fieldId,
  entryPointDescription,
  missingEntryPointWarning,
  customEntryPointNotice = null,
  isDisabled,
}: IArchiveConfigHeaderSectionProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      {entryPoint ? null : <Alert severity={"warning"}>{missingEntryPointWarning}</Alert>}

      {customEntryPointNotice && entryPoint && entryPoint !== DEFAULT_ENTRY_POINT ? (
        <Alert severity={"info"} variant={"outlined"}>
          {customEntryPointNotice}
        </Alert>
      ) : null}

      <ArchiveHeaderFields
        id={fieldId}
        header={config.header}
        entryPointDescription={entryPointDescription}
        isDisabled={isDisabled}
        onChange={(header: Nullable<ArchivePackHeader>) => editor.patchConfig({ header })}
      />
    </Stack>
  );
}
