import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { Button } from "@xrf/ui/button";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement, useCallback } from "react";

import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";

interface ISettingsAboutPendingProps {
  /** Why the section's read failed, or none while it is still reading. */
  error: Nullable<Error>;
  /** What the section says while it reads. */
  label: string;
}

/**
 * What an About section shows while its read has no answer: that it is reading, or why it failed, with a way to read
 * again.
 */
export function SettingsAboutPending({ error, label }: ISettingsAboutPendingProps): ReactElement {
  const aboutService: SettingsAboutService = useInjection(SettingsAboutService);

  const onRetry = useCallback(() => void aboutService.read(), [aboutService]);

  if (!error) {
    return <Typography variant={"caption"}>{label}</Typography>;
  }

  return (
    <Stack direction={"row"} spacing={1} className={"items-center"}>
      <Typography className={"text-error"} variant={"caption"}>
        Could not read: {error.message}
      </Typography>
      <Button size={"small"} variant={"text"} onClick={onRetry}>
        Retry
      </Button>
    </Stack>
  );
}
