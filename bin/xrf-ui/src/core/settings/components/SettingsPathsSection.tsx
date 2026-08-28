import { Box, Divider } from "@mui/material";
import { ReactElement } from "react";

import { SettingsPathRow } from "@/core/settings/components/SettingsPathRow";
import { SettingsSection } from "@/core/settings/components/SettingsSection";
import {
  IWorkspacePathDescriptor,
  PRIMARY_WORKSPACE_PATH,
  WORKSPACE_PATH_OVERRIDES,
} from "@/core/settings/lib/workspace-path";

/**
 * Where the tools look, and the places that override it.
 */
export function SettingsPathsSection(): ReactElement {
  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      <SettingsPathRow path={PRIMARY_WORKSPACE_PATH} />

      <Divider />

      <SettingsSection
        title={"Overrides"}
        description={
          "Set one only where your layout keeps something outside the game data tree. Left empty, each shows what it " +
          "derives instead."
        }
      />

      {WORKSPACE_PATH_OVERRIDES.map((it: IWorkspacePathDescriptor) => (
        <SettingsPathRow key={it.id} path={it} />
      ))}
    </Box>
  );
}
