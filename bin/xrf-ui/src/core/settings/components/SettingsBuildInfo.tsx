import { Box, Link, Stack, Typography } from "@mui/material";
import { ReactElement, useEffect, useState } from "react";

import { systemCommands } from "@/core/bindings/commands/system";
import { BuildInfo } from "@/core/bindings/types/xrf-build-info";
import { getCommitUrl, getWorkflowRunUrl } from "@/core/configs";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Which build of the application is running.
 */
export function SettingsBuildInfo(): ReactElement {
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

  if (!build) {
    return <Typography variant={"caption"}>Reading build details...</Typography>;
  }

  const rows: Array<[string, Nullable<string>, string?]> = [
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

  return (
    <Stack data-testid={"settings-build-info"} spacing={0.5}>
      {rows
        .filter(([, value]) => value)
        .map(([label, value, href]) => (
          <Box key={label} sx={{ display: "flex", gap: 1 }}>
            <Typography variant={"caption"} sx={{ minWidth: 96, opacity: 0.7 }}>
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
  );
}
