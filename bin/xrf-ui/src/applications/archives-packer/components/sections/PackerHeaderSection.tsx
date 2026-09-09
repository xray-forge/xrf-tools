import { Alert, Stack, Switch, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import {
  ArchiveHeaderEntries,
  DEFAULT_ENTRY_POINT,
  HEADER_AUTO_LOAD,
  HEADER_ENTRY_POINT,
  readHeaderFlag,
  readHeaderValue,
  writeHeaderFlag,
  writeHeaderValue,
} from "@/core/archive";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

interface IPackerHeaderSectionProps {
  config: ArchivePackConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePackConfig>) => void;
}

/**
 * The header written into the archive, which is how the engine decides where its contents mount.
 */
export function PackerHeaderSection({ config, isDisabled, onChange }: IPackerHeaderSectionProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const isAutoLoad: boolean = readHeaderFlag(config.header, HEADER_AUTO_LOAD);

  return (
    <Stack spacing={2}>
      {entryPoint ? null : (
        <Alert severity={"warning"}>
          Without an entry point the engine treats these volumes as encrypted Shadow of Chernobyl archives. Set one, or
          switch the extension to xdb under Options.
        </Alert>
      )}

      <FormRow
        label={"Entry point"}
        description={"Where the engine mounts the contents. A packed gamedata tree wants the default"}
        controlId={"packer-entry-point"}
      >
        <TextField
          id={"packer-entry-point"}
          size={"small"}
          fullWidth
          disabled={isDisabled}
          value={entryPoint ?? ""}
          placeholder={DEFAULT_ENTRY_POINT}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ header: writeHeaderValue(config.header, HEADER_ENTRY_POINT, event.target.value) })
          }
        />
      </FormRow>

      <FormRow
        label={"Mount at startup"}
        description={"Whether the engine loads these volumes on its own"}
        controlId={"packer-auto-load"}
        isInline={true}
      >
        <Switch
          id={"packer-auto-load"}
          size={"small"}
          checked={isAutoLoad}
          disabled={isDisabled}
          slotProps={{ input: { "aria-label": "Mount at startup" } }}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ header: writeHeaderFlag(config.header, HEADER_AUTO_LOAD, event.target.checked) })
          }
        />
      </FormRow>

      <ArchiveHeaderEntries
        header={config.header}
        isDisabled={isDisabled}
        onChange={(header) => onChange({ header })}
      />
    </Stack>
  );
}
