import { Stack, TextField } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ChangeEvent, ReactElement, useId } from "react";

import { ArchiveHeaderEntries } from "@/core/archive/components/ArchiveHeaderEntries";
import {
  DEFAULT_ENTRY_POINT,
  HEADER_AUTO_LOAD,
  HEADER_ENTRY_POINT,
  readHeaderFlag,
  readHeaderValue,
  writeHeaderFlag,
  writeHeaderValue,
} from "@/core/archive/lib";
import { FormRow, SwitchFormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveHeaderFieldsProps extends BaseComponentProps {
  header: Nullable<string>;
  entryPointDescription: string;
  isDisabled?: boolean;
  onChange: (header: Nullable<string>) => void;
}

/** Edits archive mounting and custom header entries without changing unrelated header values. */
export function ArchiveHeaderFields({
  "data-testid": dataTestId = "archive-header-fields",
  id,
  className,
  header,
  entryPointDescription,
  isDisabled,
  onChange,
}: IArchiveHeaderFieldsProps): ReactElement {
  const generatedId: string = useId();
  const controlId: string = id ?? generatedId;
  const entryPoint: Nullable<string> = readHeaderValue(header, HEADER_ENTRY_POINT);
  const isAutoLoad: boolean = readHeaderFlag(header, HEADER_AUTO_LOAD);

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <FormRow label={"Entry point"} description={entryPointDescription} controlId={`${controlId}-entry-point`}>
        <TextField
          id={`${controlId}-entry-point`}
          size={"small"}
          fullWidth
          disabled={isDisabled}
          value={entryPoint ?? ""}
          placeholder={DEFAULT_ENTRY_POINT}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange(writeHeaderValue(header, HEADER_ENTRY_POINT, event.target.value))
          }
        />
      </FormRow>

      <SwitchFormRow
        id={`${controlId}-auto-load`}
        label={"Mount at startup"}
        description={"Whether the engine loads these volumes on its own"}
        isChecked={isAutoLoad}
        isDisabled={isDisabled}
        onChange={(isChecked: boolean) => onChange(writeHeaderFlag(header, HEADER_AUTO_LOAD, isChecked))}
      />

      <ArchiveHeaderEntries header={header} isDisabled={isDisabled} onChange={onChange} />
    </Stack>
  );
}
