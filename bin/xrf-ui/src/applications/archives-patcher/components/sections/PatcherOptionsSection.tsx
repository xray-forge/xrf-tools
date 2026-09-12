import { Stack, Switch } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ArchiveVolumeOptionsFields } from "@/core/archive/components/ArchiveVolumeOptionsFields";
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IPatcherOptionsSectionProps extends BaseComponentProps {
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
  "data-testid": dataTestId = "patcher-options-section",
  id,
  className,
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
