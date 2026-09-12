import { ReactElement } from "react";

import { useRootProbe } from "@/core/assets/lib/use-root-probe";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField } from "@/core/ui/form/use-path-field";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IAssetRootFormRowProps extends BaseComponentProps {
  field: IPathField;
  isDisabled?: boolean;
}

/**
 * The extra tree a surface searches behind the one it opened.
 * Reports what the backend made of the directory, because a root named by hand is a guess until something confirms it.
 */
export function AssetRootFormRow({
  "data-testid": dataTestId = "asset-root-form-row",
  id,
  className,
  field,
  isDisabled,
}: IAssetRootFormRowProps): ReactElement {
  const fact: Nullable<string> = useRootProbe(field.error ? null : field.value);

  return (
    <PathFormRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Also search in"}
      description={
        "A game installation or another game data tree, searched after the one above. Leave empty to read only what " +
        "you opened."
      }
      isRequired={false}
      fact={fact}
      field={field}
      isDisabled={isDisabled}
    />
  );
}
