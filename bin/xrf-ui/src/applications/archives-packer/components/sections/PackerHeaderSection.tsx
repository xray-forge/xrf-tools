import { default as AddIcon } from "@mui/icons-material/Add";
import { default as DeleteIcon } from "@mui/icons-material/Delete";
import { Alert, Box, IconButton, Stack, Switch, TextField, Typography } from "@mui/material";
import { ChangeEvent, ReactElement, useCallback, useState } from "react";

import {
  DEFAULT_ENTRY_POINT,
  HEADER_AUTO_LOAD,
  HEADER_ENTRY_POINT,
  readHeaderEntries,
  readHeaderFlag,
  readHeaderValue,
  RESERVED_HEADER_KEYS,
  writeHeaderFlag,
  writeHeaderValue,
} from "@/applications/archives-packer/lib/pack-config";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form/FormRow";
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
  const [newKey, setNewKey] = useState<string>("");
  const [newValue, setNewValue] = useState<string>("");

  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);
  const isAutoLoad: boolean = readHeaderFlag(config.header, HEADER_AUTO_LOAD);

  const customEntries: Array<[string, string]> = readHeaderEntries(config.header).filter(
    ([key]) => !RESERVED_HEADER_KEYS.includes(key)
  );

  const trimmedKey: string = newKey.trim();
  const isDuplicateKey: boolean = Boolean(
    trimmedKey && readHeaderEntries(config.header).some(([key]) => key === trimmedKey)
  );
  const keyError: Nullable<string> = isDuplicateKey ? "That key is already in the header" : null;

  const onAddEntry = useCallback((): void => {
    if (!trimmedKey || isDuplicateKey) {
      return;
    }

    onChange({ header: writeHeaderValue(config.header, trimmedKey, newValue) });

    setNewKey("");
    setNewValue("");
  }, [config.header, isDuplicateKey, newValue, onChange, trimmedKey]);

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

      <FormRow label={"Other header values"} description={"Carried into the archive as they are"} error={keyError}>
        <Stack spacing={1}>
          {customEntries.length ? (
            customEntries.map(([key, value]) => (
              <Stack key={key} direction={"row"} spacing={1} sx={{ alignItems: "center" }}>
                <Typography variant={"body2"} className={"monospace"} sx={{ minWidth: 200, flexShrink: 0 }}>
                  {key}
                </Typography>

                <TextField
                  size={"small"}
                  fullWidth
                  disabled={isDisabled}
                  value={value}
                  slotProps={{ htmlInput: { "aria-label": `Value of ${key}` } }}
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    onChange({ header: writeHeaderValue(config.header, key, event.target.value) })
                  }
                />

                <IconButton
                  aria-label={`Remove ${key}`}
                  size={"small"}
                  disabled={isDisabled}
                  onClick={() => onChange({ header: writeHeaderValue(config.header, key, "") })}
                >
                  <DeleteIcon fontSize={"small"} />
                </IconButton>
              </Stack>
            ))
          ) : (
            <Typography variant={"body2"} color={"text.secondary"}>
              None. Importing a configuration brings its header along.
            </Typography>
          )}

          <Stack direction={"row"} spacing={1} sx={{ alignItems: "center" }}>
            <TextField
              size={"small"}
              disabled={isDisabled}
              value={newKey}
              placeholder={"key"}
              error={isDuplicateKey}
              slotProps={{ htmlInput: { "aria-label": "New header key" } }}
              sx={{ minWidth: 200, flexShrink: 0 }}
              onChange={(event: ChangeEvent<HTMLInputElement>) => setNewKey(event.target.value)}
            />

            <TextField
              size={"small"}
              fullWidth
              disabled={isDisabled}
              value={newValue}
              placeholder={"value"}
              slotProps={{ htmlInput: { "aria-label": "New header value" } }}
              onChange={(event: ChangeEvent<HTMLInputElement>) => setNewValue(event.target.value)}
            />

            <Box>
              <IconButton size={"small"} disabled={isDisabled || !trimmedKey || isDuplicateKey} onClick={onAddEntry}>
                <AddIcon fontSize={"small"} />
              </IconButton>
            </Box>
          </Stack>
        </Stack>
      </FormRow>
    </Stack>
  );
}
