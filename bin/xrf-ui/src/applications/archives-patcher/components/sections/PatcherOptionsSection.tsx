import { Stack } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ArchiveVolumeOptionsFields } from "@/core/archive/components/ArchiveVolumeOptionsFields";
import { ArchivePatchConfig } from "@/core/ipc/types/xrf-pack";
import { SwitchFormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPatcherOptionsSectionProps extends BaseComponentProps {
  config: ArchivePatchConfig;
  /** Ceiling reported by the backend, so the form does not carry its own copy of the format's limit. */
  maxVolumeSizeMegabytes: number;
  volumeSize: string;
  volumeSizeError: Nullable<string>;
  isVerifyingPayload: boolean;
  isDisabled?: boolean;
  onVolumeSizeChange: (value: string) => void;
  onVerifyingPayloadChange: (isVerifying: boolean) => void;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * How the patch is written and how carefully the comparison decides, as opposed to what goes into it.
 */
export function PatcherOptionsSection({
  "data-testid": dataTestId = "patcher-options-section",
  id,
  className,
  config,
  maxVolumeSizeMegabytes,
  volumeSize,
  volumeSizeError,
  isVerifyingPayload,
  isDisabled,
  onVolumeSizeChange,
  onVerifyingPayloadChange,
  onChange,
}: IPatcherOptionsSectionProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <ArchiveVolumeOptionsFields
        id={"patcher"}
        config={config}
        maxVolumeSizeMegabytes={maxVolumeSizeMegabytes}
        volumeSize={volumeSize}
        volumeSizeError={volumeSizeError}
        isDisabled={isDisabled}
        onVolumeSizeChange={onVolumeSizeChange}
        onChange={onChange}
      />

      <SwitchFormRow
        id={"patcher-verify-payload"}
        label={"Verify payloads"}
        description={"Confirm every checksum match by comparing the bytes, reading both sides in full"}
        isChecked={isVerifyingPayload}
        isDisabled={isDisabled}
        onChange={onVerifyingPayloadChange}
      />
    </Stack>
  );
}
