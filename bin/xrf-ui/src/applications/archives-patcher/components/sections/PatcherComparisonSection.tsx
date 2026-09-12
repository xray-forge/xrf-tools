import { Stack } from "@mui/material";
import { ReactElement } from "react";

import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { CheckboxFormRow, IPathField, PathFormRow } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IPatcherComparisonSectionProps extends BaseComponentProps {
  config: ArchivePatchConfig;
  input: IPathField;
  target: IPathField;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * What the patch is built from: the game, and optionally a tree of its own.
 */
export function PatcherComparisonSection({
  "data-testid": dataTestId = "patcher-comparison-section",
  id,
  className,
  config,
  input,
  target,
  isDisabled,
  onChange,
}: IPatcherComparisonSectionProps): ReactElement {
  const isDeliveringOwnTree: boolean = config.target !== null;

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      <PathFormRow
        isDisabled={isDisabled}
        label={"Game"}
        description={"The installation to patch; its loose gamedata is compared against its own archives"}
        field={input}
      />

      <CheckboxFormRow
        label={"Deliver another tree"}
        description={"Build the patch from a separate gamedata folder instead of the installation's loose files"}
        isChecked={isDeliveringOwnTree}
        isDisabled={isDisabled}
        onChange={(isChecked: boolean) => onChange({ target: isChecked ? (target.value ?? "") : null })}
      />

      {isDeliveringOwnTree ? (
        <PathFormRow
          isDisabled={isDisabled}
          label={"Deliver"}
          description={"The gamedata tree the patch should carry, whose children are configs, textures and meshes"}
          field={target}
        />
      ) : null}
    </Stack>
  );
}
