import { systemCommands } from "@xrf/ipc/commands/system";
import { EnvFlagReading } from "@xrf/ipc/types/xrf-env-flags";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { Stack, Typography } from "@xrf/ui/material";
import { ReactElement, useState } from "react";

import { DetailSection } from "@/core/ui/layout/DetailSection";
import { useMountEffect } from "@/lib/react/use-mount-effect";

import { IAboutRow } from "./about-row";
import { SettingsAboutRow } from "./SettingsAboutRow";
import { describeVariables } from "./SettingsVariablesSection.utils";

/**
 * The environment variables set for this run, among those the application reads.
 */
export function SettingsVariablesSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [rows, setRows] = useState<Nullable<Array<IAboutRow>>>(null);

  // Read once: the process environment does not change while it runs.
  useMountEffect(() => {
    systemCommands
      .getEnvironmentFlags()
      .then((it: Array<EnvFlagReading>) => setRows(describeVariables(it)))
      .catch((error: unknown) => log.error("Failed to read environment variables:", error));
  });

  return (
    <DetailSection
      data-testid={"settings-variables-section"}
      title={"Environment variables"}
      description={"Environment variables set for this run that change how the application behaves."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {rows === null ? (
          <Typography variant={"caption"}>Reading environment variables...</Typography>
        ) : rows.length === 0 ? (
          <Typography variant={"caption"}>None set. Every variable uses its default.</Typography>
        ) : (
          rows.map((it: IAboutRow) => <SettingsAboutRow key={it.label} label={it.label} value={it.value} />)
        )}
      </Stack>
    </DetailSection>
  );
}
