import { ReactElement } from "react";

import { IRenderSettingEntryProps, RenderSettingEntry } from "@/core/render/components/settings/RenderSettingEntry";

export interface IRenderSettingEntriesProps<G> extends Omit<IRenderSettingEntryProps<G, keyof G>, "name"> {
  /** The settings the entries set, top to bottom. */
  names: ReadonlyArray<keyof G>;
}

/** A run of one group's settings as a popover's entries, in the order named, each as its declaration says. */
export function RenderSettingEntries<G>({ names, ...props }: IRenderSettingEntriesProps<G>): ReactElement {
  return (
    <>
      {names.map((name: keyof G) => (
        <RenderSettingEntry
          key={String(name)}
          data-testid={`render-setting-entry-${String(name)}`}
          name={name}
          {...props}
        />
      ))}
    </>
  );
}
