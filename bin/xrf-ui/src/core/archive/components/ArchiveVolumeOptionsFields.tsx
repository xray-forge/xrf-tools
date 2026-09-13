import { MenuItem, Stack, TextField } from "@mui/material";
import { ChangeEvent, ReactElement, useId } from "react";

import { ARCHIVE_PACK_MODE, ARCHIVE_VOLUME_EXTENSION, ARCHIVE_VOLUME_SUFFIX } from "@/core/archive/lib";
import { ArchivePackConfig } from "@/core/ipc/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

type ArchiveVolumeOptions = Pick<ArchivePackConfig, "mode" | "volumeExtension">;

interface IArchiveVolumeOptionsFieldsProps extends BaseComponentProps {
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
  "data-testid": dataTestId = "archive-volume-options-fields",
  id,
  className,
  config,
  maxVolumeSizeMegabytes,
  volumeSize,
  volumeSizeError,
  isDisabled,
  onVolumeSizeChange,
  onChange,
}: IArchiveVolumeOptionsFieldsProps): ReactElement {
  const generatedId: string = useId();
  const controlId: string = id ?? generatedId;

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <FormRow
        label={"Compression"}
        description={"Compressed packs what the engine expects compressed and stores the rest"}
        controlId={`${controlId}-mode`}
      >
        <TextField
          id={`${controlId}-mode`}
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
        controlId={`${controlId}-volume-size`}
        error={volumeSizeError}
      >
        <TextField
          id={`${controlId}-volume-size`}
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
        controlId={`${controlId}-extension`}
      >
        <TextField
          id={`${controlId}-extension`}
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
    </Stack>
  );
}
