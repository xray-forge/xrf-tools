import { Stack, Switch } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ArchiveVolumeOptionsFields } from "@/core/archive/components/ArchiveVolumeOptionsFields";
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
      <ArchiveVolumeOptionsFields
        id={"packer"}
        config={config}
        maxVolumeSizeMegabytes={maxVolumeSizeMegabytes}
        volumeSize={volumeSize}
        volumeSizeError={volumeSizeError}
        isDisabled={isDisabled}
        onVolumeSizeChange={onVolumeSizeChange}
        onChange={onChange}
      />

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
