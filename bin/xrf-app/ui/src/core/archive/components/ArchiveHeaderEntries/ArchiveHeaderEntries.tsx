import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AddIcon } from "@xrf/ui/icons";
import { Stack, TextField, Typography } from "@xrf/ui/material";
import { ChangeEvent, ReactElement, useCallback, useState } from "react";

import { isReservedHeaderKey, isSameHeaderKey, readHeaderEntries, writeHeaderValue } from "@/core/archive/lib";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditableListItem, FormRow } from "@/core/ui/form";

interface IArchiveHeaderEntriesProps extends BaseComponentProps {
  header: Nullable<string>;
  isDisabled?: boolean;
  onChange: (header: Nullable<string>) => void;
}

/**
 * Custom header values and the draft used to append a key without replacing an existing one.
 */
export function ArchiveHeaderEntries({
  "data-testid": dataTestId = "archive-header-entries",
  id,
  className,
  header,
  isDisabled,
  onChange,
}: IArchiveHeaderEntriesProps): ReactElement {
  const [newKey, setNewKey] = useState<string>("");
  const [newValue, setNewValue] = useState<string>("");

  const entries: Array<[string, string]> = readHeaderEntries(header);
  const customEntries: Array<[string, string]> = entries.filter(([key]) => !isReservedHeaderKey(key));

  const trimmedKey: string = newKey.trim();
  const isDuplicateKey: boolean = Boolean(trimmedKey && entries.some(([key]) => isSameHeaderKey(key, trimmedKey)));
  const keyError: Nullable<string> = isDuplicateKey ? "That key is already in the header" : null;
  // Why the draft cannot be added yet, which both disables the action and says so. A blank value is how a header entry
  // is removed, so adding one would write nothing and still clear the draft.
  const addBlocker: Nullable<string> = isDisabled
    ? "Wait for the current operation to finish before editing"
    : (keyError ?? (!trimmedKey ? "Enter a header key" : null) ?? (!newValue.trim() ? "Enter a header value" : null));

  const onAddEntry = useCallback((): void => {
    if (addBlocker) {
      return;
    }

    onChange(writeHeaderValue(header, trimmedKey, newValue));

    setNewKey("");
    setNewValue("");
  }, [addBlocker, header, newValue, onChange, trimmedKey]);

  return (
    <FormRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Other header values"}
      description={"Carried into the archive as they are"}
      error={keyError}
    >
      <Stack spacing={1}>
        {customEntries.length ? (
          customEntries.map(([key, value]) => (
            <EditableListItem
              key={key}
              removeLabel={`Remove ${key}`}
              isDisabled={isDisabled}
              onRemove={() => onChange(writeHeaderValue(header, key, ""))}
            >
              <Typography className={"monospace min-w-50 shrink-0"} variant={"body2"}>
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

        <Stack className={"items-center"} direction={"row"} spacing={1}>
          <TextField
            className={"min-w-50 shrink-0"}
            size={"small"}
            disabled={isDisabled}
            value={newKey}
            placeholder={"key"}
            error={isDuplicateKey}
            slotProps={{ htmlInput: { "aria-label": "New header key" } }}
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
            description={addBlocker ?? "Add this header value"}
            icon={<AddIcon />}
            isDisabled={Boolean(addBlocker)}
            onClick={onAddEntry}
          />
        </Stack>
      </Stack>
    </FormRow>
  );
}
