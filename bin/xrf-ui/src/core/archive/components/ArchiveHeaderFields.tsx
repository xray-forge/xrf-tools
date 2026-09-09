import { Switch, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ArchiveHeaderEntries } from "@/core/archive/components/ArchiveHeaderEntries";
import {
  DEFAULT_ENTRY_POINT,
  HEADER_AUTO_LOAD,
  HEADER_ENTRY_POINT,
  readHeaderFlag,
  readHeaderValue,
  writeHeaderFlag,
  writeHeaderValue,
} from "@/core/archive/header";
import { FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

interface IArchiveHeaderFieldsProps {
  id: string;
  header: Nullable<string>;
  entryPointDescription: string;
  isDisabled?: boolean;
  onChange: (header: Nullable<string>) => void;
}

/** Edits archive mounting and custom header entries without changing unrelated header values. */
export function ArchiveHeaderFields({
  id,
  header,
  entryPointDescription,
  isDisabled,
  onChange,
}: IArchiveHeaderFieldsProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(header, HEADER_ENTRY_POINT);
  const isAutoLoad: boolean = readHeaderFlag(header, HEADER_AUTO_LOAD);

  return (
    <>
      <FormRow label={"Entry point"} description={entryPointDescription} controlId={`${id}-entry-point`}>
        <TextField
          id={`${id}-entry-point`}
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

      <FormRow
        label={"Mount at startup"}
        description={"Whether the engine loads these volumes on its own"}
        controlId={`${id}-auto-load`}
        isInline={true}
      >
        <Switch
          id={`${id}-auto-load`}
          size={"small"}
          checked={isAutoLoad}
          disabled={isDisabled}
          slotProps={{ input: { "aria-label": "Mount at startup" } }}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange(writeHeaderFlag(header, HEADER_AUTO_LOAD, event.target.checked))
          }
        />
      </FormRow>

      <ArchiveHeaderEntries header={header} isDisabled={isDisabled} onChange={onChange} />
    </>
  );
}
