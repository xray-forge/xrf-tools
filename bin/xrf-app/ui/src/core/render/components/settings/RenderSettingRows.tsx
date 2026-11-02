import { ReactElement } from "react";

import { IRenderSettingRowProps, RenderSettingRow } from "@/core/render/components/settings/RenderSettingRow";

export interface IRenderSettingRowsProps<G> extends Omit<IRenderSettingRowProps<G, keyof G>, "name"> {
  /** The settings the rows set, top to bottom. */
  names: ReadonlyArray<keyof G>;
}

/** A run of one group's settings as rows in the settings, in the order named, each as its declaration says. */
export function RenderSettingRows<G>({ names, ...props }: IRenderSettingRowsProps<G>): ReactElement {
  return (
    <>
      {names.map((name: keyof G) => (
        <RenderSettingRow
          key={String(name)}
          data-testid={`render-setting-row-${String(name)}`}
          name={name}
          {...props}
        />
      ))}
    </>
  );
}
