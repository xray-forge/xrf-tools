import { MenuItem, Stack, Switch, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import {
  ARCHIVE_PACK_MODE,
  ARCHIVE_VOLUME_EXTENSION,
  ARCHIVE_VOLUME_SUFFIX,
} from "@/applications/archives-packer/lib/pack-config";
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

interface IPatcherOptionsSectionProps {
  config: ArchivePatchConfig;
  /** Ceiling reported by the backend, so the form does not carry its own copy of the format's limit. */
  maxVolumeSizeMegabytes: number;
  volumeSize: string;
  volumeSizeError: Nullable<string>;
  isVerifyingPayload: boolean;
  isForced: boolean;
  isDisabled?: boolean;
  onVolumeSizeChange: (value: string) => void;
  onVerifyingPayloadChange: (isVerifying: boolean) => void;
  onForcedChange: (isForced: boolean) => void;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * How the patch is written and how carefully the comparison decides, as opposed to what goes into it.
 */
export function PatcherOptionsSection({
  config,
  maxVolumeSizeMegabytes,
  volumeSize,
  volumeSizeError,
  isVerifyingPayload,
  isForced,
  isDisabled,
  onVolumeSizeChange,
  onVerifyingPayloadChange,
  onForcedChange,
  onChange,
}: IPatcherOptionsSectionProps): ReactElement {
  return (
    <Stack spacing={2}>
      <FormRow
        label={"Compression"}
        description={"Compressed packs what the engine expects compressed and stores the rest"}
        controlId={"patcher-mode"}
      >
        <TextField
          id={"patcher-mode"}
          size={"small"}
          fullWidth
          select
          disabled={isDisabled}
          value={config.mode}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ mode: event.target.value as ArchivePatchConfig["mode"] })
          }
        >
          <MenuItem value={ARCHIVE_PACK_MODE.Compress}>Compressed</MenuItem>
          <MenuItem value={ARCHIVE_PACK_MODE.Store}>Stored only</MenuItem>
        </TextField>
      </FormRow>

      <FormRow
        label={"Volume size"}
        description={`Megabytes before a new volume starts, up to ${maxVolumeSizeMegabytes}`}
        controlId={"patcher-volume-size"}
        error={volumeSizeError}
      >
        <TextField
          id={"patcher-volume-size"}
          size={"small"}
          fullWidth
          type={"number"}
          disabled={isDisabled}
          value={volumeSize}
          error={Boolean(volumeSizeError)}
          placeholder={String(maxVolumeSizeMegabytes)}
          slotProps={{ htmlInput: { min: 1, max: maxVolumeSizeMegabytes } }}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onVolumeSizeChange(event.target.value)}
        />
      </FormRow>

      <FormRow
        label={"Extension"}
        description={"An xdb archive is never mistaken for an encrypted Shadow of Chernobyl one"}
        controlId={"patcher-extension"}
      >
        <TextField
          id={"patcher-extension"}
          size={"small"}
          fullWidth
          select
          disabled={isDisabled}
          value={config.volumeExtension}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ volumeExtension: event.target.value as ArchivePatchConfig["volumeExtension"] })
          }
        >
          <MenuItem value={ARCHIVE_VOLUME_EXTENSION.Db}>{ARCHIVE_VOLUME_SUFFIX.Db}</MenuItem>
          <MenuItem value={ARCHIVE_VOLUME_EXTENSION.Xdb}>{ARCHIVE_VOLUME_SUFFIX.Xdb}</MenuItem>
        </TextField>
      </FormRow>

      <FormRow
        label={"Verify payloads"}
        description={"Confirm every checksum match by comparing the bytes, reading both sides in full"}
        controlId={"patcher-verify-payload"}
        isInline
      >
        <Switch
          id={"patcher-verify-payload"}
          disabled={isDisabled}
          checked={isVerifyingPayload}
          slotProps={{ input: { "aria-label": "Verify payloads" } }}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onVerifyingPayloadChange(event.target.checked)}
        />
      </FormRow>

      <FormRow
        label={"Replace existing output volumes"}
        description={"Publishing over a set already in the output cannot be undone if it fails partway"}
        controlId={"patcher-force"}
        isInline
      >
        <Switch
          id={"patcher-force"}
          disabled={isDisabled}
          checked={isForced}
          slotProps={{ input: { "aria-label": "Replace existing output volumes" } }}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onForcedChange(event.target.checked)}
        />
      </FormRow>
    </Stack>
  );
}
