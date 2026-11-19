import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { Checkbox, FormControlLabel } from "@xrf/ui/form/control";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement, ReactNode } from "react";

interface IArchiveRunConfirmSummaryProps extends BaseComponentProps {
  /** Volumes of this set the output already holds, which a run refuses to replace unasked; none asks nothing. */
  publishedVolumes: ReadonlyArray<string>;
  /** What replacing them costs this run, said above the choice to replace them. */
  replaceWarning: ReactNode;
  isForced: boolean;
  onForceChange: (isForced: boolean) => void;
  /** The tool's rows: what the run reads, writes and how. */
  children: ReactNode;
}

/**
 * What an archive run is about to do, read from its configuration. Replacing an output that already holds this set is
 * asked for here, not assumed.
 */
export function ArchiveRunConfirmSummary({
  "data-testid": dataTestId = "archive-run-confirm-summary",
  id,
  className,
  publishedVolumes,
  replaceWarning,
  isForced,
  onForceChange,
  children,
}: IArchiveRunConfirmSummaryProps): ReactElement {
  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={1.5}>
      {children}

      {publishedVolumes.length ? (
        <Alert severity={"warning"}>
          <Stack spacing={0.5}>
            <Typography variant={"body2"}>{replaceWarning}</Typography>

            <FormControlLabel
              control={
                <Checkbox size={"small"} checked={isForced} onChange={(event) => onForceChange(event.target.checked)} />
              }
              label={<Typography variant={"body2"}>Replace them</Typography>}
            />
          </Stack>
        </Alert>
      ) : null}
    </Stack>
  );
}
