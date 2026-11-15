import { EnvFlagReading } from "@xrf/ipc/types/xrf-env-flags";

import { IAboutRow } from "./about-row";

/**
 * @param readings - The state of every environment variable the backend reads.
 * @returns A row for each set variable, with the default that applies when its value is invalid.
 */
export function describeVariables(readings: Array<EnvFlagReading>): Array<IAboutRow> {
  return readings.flatMap((it: EnvFlagReading) =>
    it.value === null
      ? []
      : [
          {
            label: it.name,
            value: it.applied === null ? `${it.value} (not valid, the default applies: ${it.default})` : it.value,
          },
        ]
  );
}
