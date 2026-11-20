import { Optional } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FormRow } from "@xrf/ui/form/row";
import { TextField } from "@xrf/ui/input";
import { Stack } from "@xrf/ui/layout/stack";
import { ChangeEvent, ReactElement, ReactNode } from "react";

import { IArchiveConfigEditor, IArchiveEditedConfig } from "@/core/archive/services/archive-config-editor.service";
import { IPathField, PathFormRow } from "@/core/ui/form";

interface IArchiveConfigOutputSectionProps extends BaseComponentProps {
  editor: IArchiveConfigEditor;
  /** The editor's open configuration. */
  config: IArchiveEditedConfig;
  /** Prefix of the fields' control ids. */
  fieldId: string;
  destination: IPathField;
  destinationDescription: string;
  nameLabel: string;
  nameDescription: string;
  namePlaceholder?: Optional<string>;
  isDisabled?: boolean;
  /** The tool's own path rows, above the output, such as where a run reads from. */
  children?: ReactNode;
}

/**
 * Where the volumes land and what they are called. The paths belong to the run rather than to a configuration file,
 * so they are never written to one.
 */
export function ArchiveConfigOutputSection({
  "data-testid": dataTestId = "archive-config-output-section",
  id,
  className,
  editor,
  config,
  fieldId,
  destination,
  destinationDescription,
  nameLabel,
  nameDescription,
  namePlaceholder,
  isDisabled,
  children,
}: IArchiveConfigOutputSectionProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      {children}

      <PathFormRow isDisabled={isDisabled} label={"Output"} description={destinationDescription} field={destination} />

      <FormRow label={nameLabel} description={nameDescription} controlId={`${fieldId}-name`}>
        <TextField
          id={`${fieldId}-name`}
          size={"small"}
          fullWidth
          disabled={isDisabled}
          value={config.name}
          placeholder={namePlaceholder}
          onChange={(event: ChangeEvent<HTMLInputElement>) => editor.patchConfig({ name: event.target.value })}
        />
      </FormRow>
    </Stack>
  );
}
