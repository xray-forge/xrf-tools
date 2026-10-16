import { Stack, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useState } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { EnvFlagReading } from "@/core/ipc/types/xrf-env-flags";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { Logger, useLogger } from "@/lib/logging";
import { useMountEffect } from "@/lib/react";

import { IAboutRow } from "./about-row";
import { SettingsAboutRow } from "./SettingsAboutRow";
import { describeVariables } from "./SettingsVariablesSection.utils";

/**
 * The environment variables this run was started with, of those the application reads.
 */
export function SettingsVariablesSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [rows, setRows] = useState<Nullable<Array<IAboutRow>>>(null);

  // Read once: the environment is the process's own and does not change while it runs.
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
      description={"The variables this run was started with that change what the application does."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {rows === null ? (
          <Typography variant={"caption"}>Reading environment variables...</Typography>
        ) : rows.length === 0 ? (
          <Typography variant={"caption"}>None set: every one has its default.</Typography>
        ) : (
          rows.map((it: IAboutRow) => <SettingsAboutRow key={it.label} label={it.label} value={it.value} />)
        )}
      </Stack>
    </DetailSection>
  );
}
