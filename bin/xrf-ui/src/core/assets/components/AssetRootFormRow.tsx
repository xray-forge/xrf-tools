import { ReactElement } from "react";

import { useRootProbe } from "@/core/assets/lib/use-root-probe";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField } from "@/core/ui/form/use-path-field";
import { Nullable } from "@/lib/types/general";

interface IAssetRootFormRowProps {
  field: IPathField;
  isDisabled?: boolean;
}

/**
 * The extra tree a surface searches behind the one it opened.
 *
 * A layered install is the normal arrangement - a mod tree carries what it changed and the game answers for the rest -
 * and this is where that arrangement is stated. Named at the point of use rather than configured once for every tool,
 * so what a document resolved against is a parameter of opening it and not an ambient setting somewhere else.
 *
 * Reports what the backend made of the directory, because a root named by hand is a guess until something confirms it.
 */
export function AssetRootFormRow({ field, isDisabled }: IAssetRootFormRowProps): ReactElement {
  const fact: Nullable<string> = useRootProbe(field.error ? null : field.value);

  return (
    <PathFormRow
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
