import { EnvFlagReading } from "@/core/ipc/types/xrf-env-flags";

import { IAboutRow } from "./about-row";

/**
 * @param readings - Every environment variable the backend reads, as this run found it.
 * @returns A row for each one set: its value, and where it does not take the value, the default that applies instead.
 */
export function describeVariables(readings: Array<EnvFlagReading>): Array<IAboutRow> {
  return readings.flatMap((it: EnvFlagReading) =>
    it.value === null
      ? []
      : [
          {
            label: it.name,
            value: it.applied === null ? `${it.value} (not taken; default: ${it.default})` : it.value,
          },
        ]
  );
}
