import { useEffect, useState } from "react";

import { describeRootProbe } from "@/core/assets/lib/root-probe";
import { assetsCommands } from "@/core/bindings/commands/assets";
import { XrayRootProbe } from "@/core/bindings/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

/**
 * What the backend makes of a directory a person named as an asset root.
 *
 * Asked for because naming a root by hand is a guess until something confirms it: a tree missing its `textures`
 * directory, or an installation named one level too high, both look right in a text field and resolve nothing. This is
 * the reply, in one line, and it is the reason the probe exists at all.
 *
 * A failed check reports nothing rather than a problem, the way every other path check here treats one: not being able
 * to ask is not the same as the answer being no.
 *
 * @param path - The directory to describe, or `null` when the field is empty.
 * @returns What it is, or `null` while unknown, unset, or unanswerable.
 */
export function useRootProbe(path: Nullable<string>): Nullable<string> {
  const [fact, setFact] = useState<Nullable<string>>(null);

  useEffect(() => {
    if (!path) {
      setFact(null);

      return;
    }

    let isCurrent: boolean = true;

    assetsCommands
      .probeRoot(path)
      .then((probe: XrayRootProbe) => isCurrent && setFact(describeRootProbe(probe)))
      .catch(() => isCurrent && setFact(null));

    return () => {
      isCurrent = false;
    };
  }, [path]);

  return fact;
}
