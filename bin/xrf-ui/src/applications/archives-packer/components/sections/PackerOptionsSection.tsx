import { MenuItem, Stack, Switch, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import {
  ARCHIVE_PACK_MODE,
  ARCHIVE_VOLUME_EXTENSION,
  ARCHIVE_VOLUME_SUFFIX,
} from "@/applications/archives-packer/lib/pack-config";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

interface IPackerOptionsSectionProps {
  config: ArchivePackConfig;
  /** Ceiling reported by the packer, so the form does not carry its own copy of the format's limit. */
  maxVolumeSizeMegabytes: number;
  volumeSizeError: Nullable<string>;
  volumeSize: string;
  isDisabled?: boolean;
  onVolumeSizeChange: (value: string) => void;
  onChange: (patch: Partial<ArchivePackConfig>) => void;
}

/**
 * How the archive is written, as opposed to what goes into it.
 */
export function PackerOptionsSection({
  config,
  maxVolumeSizeMegabytes,
  volumeSize,
  volumeSizeError,
  isDisabled,
  onVolumeSizeChange,
  onChange,
}: IPackerOptionsSectionProps): ReactElement {
  return (
    <Stack spacing={2}>
      <FormRow
        label={"Compression"}
        description={"Compressed packs what the engine expects compressed and stores the rest"}
        controlId={"packer-mode"}
      >
        <TextField
          id={"packer-mode"}
          size={"small"}
          fullWidth
          select
          disabled={isDisabled}
          value={config.mode}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ mode: event.target.value as ArchivePackConfig["mode"] })
          }
        >
          <MenuItem value={ARCHIVE_PACK_MODE.Compress}>Compressed</MenuItem>
          <MenuItem value={ARCHIVE_PACK_MODE.Store}>Stored only</MenuItem>
        </TextField>
      </FormRow>

      <FormRow
        label={"Volume size"}
        description={`Megabytes before a new volume starts, up to ${maxVolumeSizeMegabytes}`}
        controlId={"packer-volume-size"}
        error={volumeSizeError}
      >
        <TextField
          id={"packer-volume-size"}
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
        controlId={"packer-extension"}
      >
        <TextField
          id={"packer-extension"}
          size={"small"}
          fullWidth
          select
          disabled={isDisabled}
          value={config.volumeExtension}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ volumeExtension: event.target.value as ArchivePackConfig["volumeExtension"] })
          }
        >
          <MenuItem value={ARCHIVE_VOLUME_EXTENSION.Db}>{ARCHIVE_VOLUME_SUFFIX.Db}</MenuItem>
          <MenuItem value={ARCHIVE_VOLUME_EXTENSION.Xdb}>{ARCHIVE_VOLUME_SUFFIX.Xdb}</MenuItem>
        </TextField>
      </FormRow>

      <FormRow
        label={"Skip editor leftovers"}
        description={"Drops the sources and intermediates a game build never reads, as xrCompress does"}
        controlId={"packer-skip-list"}
        isInline
      >
        <Switch
          id={"packer-skip-list"}
          disabled={isDisabled}
          checked={config.isWithSkipList}
          slotProps={{ input: { "aria-label": "Skip editor and source leftovers" } }}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onChange({ isWithSkipList: event.target.checked })}
        />
      </FormRow>
    </Stack>
  );
}
