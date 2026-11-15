import { useInjection } from "@wirestate/react";
import { EnvFlagReading } from "@xrf/ipc/types/xrf-env-flags";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";

import { IAboutRow } from "./about-row";
import { describeVariables } from "./SettingsVariablesSection.utils";

/**
 * The environment variables set for this run, among those the application reads.
 */
export function SettingsVariablesSection(): ReactElement {
  const aboutService: SettingsAboutService = useInjection(SettingsAboutService);

  const variables: Nullable<Array<EnvFlagReading>> = aboutService.variables.value;
  const rows: Nullable<Array<IAboutRow>> = variables ? describeVariables(variables) : null;

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
          rows.map((it: IAboutRow) => (
            <EditorPanelProperty key={it.label} label={it.label} value={it.value} isMonospace isInline />
          ))
        )}
      </Stack>
    </DetailSection>
  );
}
