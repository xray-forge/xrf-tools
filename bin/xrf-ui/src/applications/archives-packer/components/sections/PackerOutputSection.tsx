import { Stack, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow, IPathField, PathFormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPackerOutputSectionProps extends BaseComponentProps {
  config: ArchivePackConfig;
  source: IPathField;
  destination: IPathField;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePackConfig>) => void;
}

/**
 * Where the archive is built from and where it lands, which belong to the run rather than to a
 * configuration file and so are never written to one.
 */
export function PackerOutputSection({
  "data-testid": dataTestId = "packer-output-section",
  id,
  className,
  config,
  source,
  destination,
  isDisabled,
  onChange,
}: IPackerOutputSectionProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <PathFormRow
        isDisabled={isDisabled}
        label={"Source"}
        description={"Directory to pack. Every rule below is relative to it"}
        field={source}
      />

      <PathFormRow
        isDisabled={isDisabled}
        label={"Output"}
        description={"Directory the volumes are written into, overwriting volumes of the same name"}
        field={destination}
      />

      <FormRow
        label={"Name"}
        description={"Base name of the volumes: one volume is <name>.db, several are <name>.db0 and up"}
        controlId={"packer-name"}
      >
        <TextField
          id={"packer-name"}
          size={"small"}
          fullWidth
          disabled={isDisabled}
          value={config.name}
          placeholder={"gamedata"}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onChange({ name: event.target.value })}
        />
      </FormRow>
    </Stack>
  );
}
