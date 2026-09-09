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
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

interface IPatcherHeaderSectionProps {
  config: ArchivePatchConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * The header written into the patch volumes, which is how the engine decides where they mount.
 */
export function PatcherHeaderSection({ config, isDisabled, onChange }: IPatcherHeaderSectionProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const isAutoLoad: boolean = readHeaderFlag(config.header, HEADER_AUTO_LOAD);

  return (
    <Stack spacing={2}>
      {entryPoint ? null : (
        <Alert severity={"warning"}>
          The engine reads the entry point without checking whether it is there, so volumes without one stop the game on
          load. Publishing refuses a header that omits it.
        </Alert>
      )}

      {entryPoint && entryPoint !== DEFAULT_ENTRY_POINT ? (
        <Alert severity={"info"} variant={"outlined"}>
          A patch overrides entries by name, so it has to mount where the release it patches did. Every shipped
          configuration uses {DEFAULT_ENTRY_POINT}.
        </Alert>
      ) : null}

      <FormRow
        label={"Entry point"}
        description={"Where the engine mounts the contents. A patch over a gamedata release wants the default"}
        controlId={"patcher-entry-point"}
      >
        <TextField
          id={"patcher-entry-point"}
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
        controlId={"patcher-auto-load"}
        isInline={true}
      >
        <Switch
          id={"patcher-auto-load"}
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
