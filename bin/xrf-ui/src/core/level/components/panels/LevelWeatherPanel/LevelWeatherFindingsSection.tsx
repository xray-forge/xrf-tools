import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { EnvironmentFinding } from "@/core/ipc/types/xrf-environment";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherFindingsSectionProps extends BaseComponentProps {
  /** The cycle's config, as a logical path. */
  file: string;
  findings: ReadonlyArray<EnvironmentFinding>;
}

/**
 * What is wrong in the playing cycle's config, each where it is.
 */
export function LevelWeatherFindingsSection({
  "data-testid": dataTestId = "level-weather-findings-section",
  id,
  className,
  file,
  findings,
}: ILevelWeatherFindingsSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Findings"} caption={file}>
      {findings.length ? (
        <ul className={"flex flex-col gap-2"}>
          {findings.map((finding: EnvironmentFinding, index: number) => (
            <li key={index} className={"min-w-0"}>
              <Typography className={"block wrap-anywhere"} variant={"body2"}>
                {finding.message}
              </Typography>
              <Typography className={"block wrap-anywhere text-text-secondary"} variant={"caption"}>
                {[finding.rule, finding.section ? `[${finding.section}]` : null, finding.key]
                  .filter(Boolean)
                  .join(" · ")}
              </Typography>
            </li>
          ))}
        </ul>
      ) : (
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          Nothing wrong in its config.
        </Typography>
      )}
    </EditorPanelSection>
  );
}
