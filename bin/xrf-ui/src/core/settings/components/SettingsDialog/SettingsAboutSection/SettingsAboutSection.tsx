import { Box, Link, Stack, Typography } from "@mui/material";
import { ReactElement, useEffect, useState } from "react";

import { systemCommands } from "@/core/bindings/commands/system";
import { BuildInfo } from "@/core/bindings/types/xrf-build-info";
import { getCommitUrl, getWorkflowRunUrl } from "@/core/configs";
import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Room the labels keep, so the values line up in one column. */
const LABEL_WIDTH: number = 96;

/** One row of the build: what it is, what it says, and where it can be followed to. */
type TBuildRow = [string, Nullable<string>, string?];

/**
 * @param build - Build the backend reported.
 * @returns Every row worth stating about it, in the order they read.
 */
function describeBuild(build: BuildInfo): Array<TBuildRow> {
  return [
    ["Version", `${build.version} (${build.kind})`],
    [
      "Commit",
      build.commit ? `${build.commit.slice(0, 7)}${build.isDirty ? " (dirty)" : ""}` : null,
      build.commit ? getCommitUrl(build.commit) : undefined,
    ],
    ["Branch", build.reference],
    ["Built", build.builtAt],
    ["Target", build.target],
    ["Compiler", build.rustc],
    ["Optimization", build.optimization],
    ["Workflow", build.runId, build.runId ? getWorkflowRunUrl(build.runId) : undefined],
  ];
}

/**
 * Which build of the application is running.
 */
export function SettingsAboutSection({
  "data-testid": dataTestId = "settings-about-section",
  className,
  id,
}: BaseComponentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [build, setBuild] = useState<Nullable<BuildInfo>>(null);


  useEffect(() => {
    systemCommands
      .getBuildInfo()
      .then((it) => {
        log.info("Received build info:", it);

        setBuild(it);
      })
      .catch((error: unknown) => log.error("Failed to read build info:", error));
  }, [log]);

  return (
    <SettingsSection
      data-testid={dataTestId}
      className={className}
      id={id}
      title={"Build"}
      description={"Which build of the application is running, and where it came from."}
    >
      <Stack data-testid={"settings-build-info"} spacing={0.5} sx={{ marginTop: 1 }}>
        {build ? null : <Typography variant={"caption"}>Reading build details...</Typography>}

        {(build ? describeBuild(build) : [])
          .filter(([, value]: TBuildRow) => value)
          .map(([label, value, href]: TBuildRow) => (
            <Box key={label} sx={{ display: "flex", gap: 1 }}>
              <Typography variant={"caption"} sx={{ minWidth: LABEL_WIDTH, opacity: 0.7 }}>
                {label}
              </Typography>

              {href ? (
                <Link
                  variant={"caption"}
                  href={href}
                  target={"_blank"}
                  rel={"noreferrer"}
                  sx={{ fontFamily: "monospace", wordBreak: "break-all" }}
                >
                  {value}
                </Link>
              ) : (
                <Typography variant={"caption"} sx={{ fontFamily: "monospace", wordBreak: "break-all" }}>
                  {value}
                </Typography>
              )}
            </Box>
          ))}
      </Stack>
    </SettingsSection>
  );
}
