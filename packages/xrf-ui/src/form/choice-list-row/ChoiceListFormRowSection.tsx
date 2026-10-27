import { Optional } from "@xrf/types";
import { ReactElement, ReactNode, useId } from "react";

import { ListSubheader } from "#/material/list";

interface IChoiceListFormRowSectionProps {
  /** The heading over the options, which none leaves bare. */
  label: Optional<string>;
  children: ReactNode;
}

/**
 * A run of a choice list's options as one group under its heading, or the options alone where they have none.
 */
export function ChoiceListFormRowSection({ label, children }: IChoiceListFormRowSectionProps): ReactElement {
  const headingId: string = useId();

  if (label === undefined) {
    return <>{children}</>;
  }

  return (
    // A rule above every group but the first, apart from the heading's own so it scrolls away while the heading sticks.
    <li role={"presentation"} className={"border-divider [&:not(:first-child)]:border-t"}>
      <ul role={"group"} aria-labelledby={headingId} className={"p-0"}>
        <ListSubheader
          id={headingId}
          className={"border-b border-divider text-xs leading-7 font-semibold tracking-wider uppercase"}
        >
          {label}
        </ListSubheader>

        {children}
      </ul>
    </li>
  );
}
