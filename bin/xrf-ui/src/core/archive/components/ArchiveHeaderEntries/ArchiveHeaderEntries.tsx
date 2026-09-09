import { default as AddIcon } from "@mui/icons-material/Add";
import { Stack, TextField, Typography } from "@mui/material";
import { ChangeEvent, ReactElement, useCallback, useState } from "react";

import { readHeaderEntries, RESERVED_HEADER_KEYS, writeHeaderValue } from "@/core/archive";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditableListItem, FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

interface IArchiveHeaderEntriesProps {
  header: Nullable<string>;
  isDisabled?: boolean;
  onChange: (header: Nullable<string>) => void;
}

/**
 * Custom header values and the draft used to append a key without replacing an existing one.
 */
export function ArchiveHeaderEntries({ header, isDisabled, onChange }: IArchiveHeaderEntriesProps): ReactElement {
  const [newKey, setNewKey] = useState<string>("");
  const [newValue, setNewValue] = useState<string>("");

  const entries: Array<[string, string]> = readHeaderEntries(header);
  const customEntries: Array<[string, string]> = entries.filter(([key]) => !RESERVED_HEADER_KEYS.includes(key));

  const trimmedKey: string = newKey.trim();
  const isDuplicateKey: boolean = Boolean(trimmedKey && entries.some(([key]) => key === trimmedKey));
  const keyError: Nullable<string> = isDuplicateKey ? "That key is already in the header" : null;

  const onAddEntry = useCallback((): void => {
    if (isDisabled || !trimmedKey || isDuplicateKey) {
      return;
    }

    onChange(writeHeaderValue(header, trimmedKey, newValue));

    setNewKey("");
    setNewValue("");
  }, [header, isDisabled, isDuplicateKey, newValue, onChange, trimmedKey]);

  return (
    <FormRow label={"Other header values"} description={"Carried into the archive as they are"} error={keyError}>
      <Stack spacing={1}>
        {customEntries.length ? (
          customEntries.map(([key, value]) => (
            <EditableListItem
              key={key}
              removeLabel={`Remove ${key}`}
              isDisabled={isDisabled}
              onRemove={() => onChange(writeHeaderValue(header, key, ""))}
            >
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
                  onChange(writeHeaderValue(header, key, event.target.value))
                }
              />
            </EditableListItem>
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

          <EditorIconAction
            label={"Add header value"}
            description={
              isDisabled
                ? "Wait for the current operation to finish before editing"
                : (keyError ?? (!trimmedKey ? "Enter a header key" : "Add this header value"))
            }
            icon={<AddIcon />}
            isDisabled={isDisabled || !trimmedKey || isDuplicateKey}
            onClick={onAddEntry}
          />
        </Stack>
      </Stack>
    </FormRow>
  );
}
