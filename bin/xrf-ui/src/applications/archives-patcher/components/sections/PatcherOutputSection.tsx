import { Stack, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ArchivePatchConfig } from "@/core/ipc/types/xrf-pack";
import { FormRow, IPathField, PathFormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPatcherOutputSectionProps extends BaseComponentProps {
  config: ArchivePatchConfig;
  destination: IPathField;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * Where the volumes land and what they are called.
 */
export function PatcherOutputSection({
  "data-testid": dataTestId = "patcher-output-section",
  id,
  className,
  config,
  destination,
  isDisabled,
  onChange,
}: IPatcherOutputSectionProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <PathFormRow
        isDisabled={isDisabled}
        label={"Output"}
        description={"Directory the patch volumes are written into, outside the game"}
        field={destination}
      />

      <FormRow
        label={"Volume name"}
        description={"Volumes are written as <name>.db0, <name>.db1 and so on"}
        controlId={"patcher-name"}
      >
        <TextField
          id={"patcher-name"}
          size={"small"}
          fullWidth
          disabled={isDisabled}
          value={config.name}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onChange({ name: event.target.value })}
        />
      </FormRow>
    </Stack>
  );
}
