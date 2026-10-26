import { ReactElement } from "react";

import { EConfigsDialect } from "@/core/ltx/lib/dialect";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form";

const DIALECT_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EConfigsDialect>> = [
  { value: EConfigsDialect.LTX, label: "LTX" },
  { value: EConfigsDialect.DLTX, label: "DLTX" },
];

interface IConfigsDialectFormRowProps {
  isDltx: boolean;
  isDisabled?: boolean;
  onChange: (isDltx: boolean) => void;
}

/** Select the rules used to read a configuration tree. */
export function ConfigsDialectFormRow({ isDltx, isDisabled, onChange }: IConfigsDialectFormRowProps): ReactElement {
  return (
    <ChoiceFormRow
      label={"Dialect"}
      description={"LTX uses standard rules; DLTX applies the mod_*.ltx patches used by Monolith and Anomaly"}
      options={DIALECT_OPTIONS}
      value={isDltx ? EConfigsDialect.DLTX : EConfigsDialect.LTX}
      isDisabled={isDisabled}
      onChange={(dialect) => onChange(dialect === EConfigsDialect.DLTX)}
    />
  );
}
