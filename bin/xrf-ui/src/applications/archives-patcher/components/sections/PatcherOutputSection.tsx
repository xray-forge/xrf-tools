import { Stack, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow, IPathField, PathFormRow } from "@/core/ui/form";

interface IPatcherOutputSectionProps {
  config: ArchivePatchConfig;
  destination: IPathField;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * Where the volumes land and what they are called.
 */
export function PatcherOutputSection({
  config,
  destination,
  isDisabled,
  onChange,
}: IPatcherOutputSectionProps): ReactElement {
  return (
    <Stack spacing={2}>
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
