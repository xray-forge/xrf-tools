import { MenuItem, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ARCHIVE_PACK_MODE, ARCHIVE_VOLUME_EXTENSION, ARCHIVE_VOLUME_SUFFIX } from "@/core/archive/volume-options";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { Nullable } from "@/lib/types/general";

type ArchiveVolumeOptions = Pick<ArchivePackConfig, "mode" | "volumeExtension">;

interface IArchiveVolumeOptionsFieldsProps {
  id: string;
  config: ArchiveVolumeOptions;
  maxVolumeSizeMegabytes: number;
  volumeSize: string;
  volumeSizeError: Nullable<string>;
  isDisabled?: boolean;
  onVolumeSizeChange: (value: string) => void;
  onChange: (patch: Partial<ArchiveVolumeOptions>) => void;
}

/** Common write options for packed archives and patches. */
export function ArchiveVolumeOptionsFields({
  id,
  config,
  maxVolumeSizeMegabytes,
  volumeSize,
  volumeSizeError,
  isDisabled,
  onVolumeSizeChange,
  onChange,
}: IArchiveVolumeOptionsFieldsProps): ReactElement {
  return (
    <>
      <FormRow
        label={"Compression"}
        description={"Compressed packs what the engine expects compressed and stores the rest"}
        controlId={`${id}-mode`}
      >
        <TextField
          id={`${id}-mode`}
          size={"small"}
          fullWidth
          select
          disabled={isDisabled}
          value={config.mode}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ mode: event.target.value as ArchiveVolumeOptions["mode"] })
          }
        >
          <MenuItem value={ARCHIVE_PACK_MODE.Compress}>Compressed</MenuItem>
          <MenuItem value={ARCHIVE_PACK_MODE.Store}>Stored only</MenuItem>
        </TextField>
      </FormRow>

      <FormRow
        label={"Volume size"}
        description={`Megabytes before a new volume starts, up to ${maxVolumeSizeMegabytes}`}
        controlId={`${id}-volume-size`}
        error={volumeSizeError}
      >
        <TextField
          id={`${id}-volume-size`}
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
        controlId={`${id}-extension`}
      >
        <TextField
          id={`${id}-extension`}
          size={"small"}
          fullWidth
          select
          disabled={isDisabled}
          value={config.volumeExtension}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            onChange({ volumeExtension: event.target.value as ArchiveVolumeOptions["volumeExtension"] })
          }
        >
          <MenuItem value={ARCHIVE_VOLUME_EXTENSION.Db}>{ARCHIVE_VOLUME_SUFFIX.Db}</MenuItem>
          <MenuItem value={ARCHIVE_VOLUME_EXTENSION.Xdb}>{ARCHIVE_VOLUME_SUFFIX.Xdb}</MenuItem>
        </TextField>
      </FormRow>
    </>
  );
}
