import { Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { WebviewOptions, WebviewOptionsStatus } from "@/core/ipc/types/xrf-app";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { Logger, useLogger } from "@/lib/logging";
import { useMountEffect } from "@/lib/react";

import { isRestartPending } from "./SettingsWebviewOptionsSection.utils";

/**
 * The webview browser's optional capabilities, which the backend keeps and applies as the application next starts.
 */
export function SettingsWebviewOptionsSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [status, setStatus] = useState<Nullable<WebviewOptionsStatus>>(null);

  const onRead = useCallback(() => {
    systemCommands
      .getWebviewOptions()
      .then(setStatus)
      .catch((error: unknown) => log.error("Failed to read the webview options:", error));
  }, [log]);

  // Shown chosen at once; what the backend kept replaces it, and a choice it could not keep is read back.
  const onChange = useCallback(
    (change: Partial<WebviewOptions>) => {
      if (!status) {
        return;
      }

      const chosen: WebviewOptions = { ...status.chosen, ...change };

      setStatus({ ...status, chosen });

      systemCommands
        .setWebviewOptions(chosen)
        .then(setStatus)
        .catch((error: unknown) => {
          log.error("Failed to keep the webview options:", error);
          onRead();
        });
    },
    [log, onRead, status]
  );

  useMountEffect(onRead);

  return (
    <DetailSection
      data-testid={"settings-webview-options-section"}
      title={"Browser"}
      description={"Capabilities the webview's browser starts with, taken as the application next starts."}
      fact={status && isRestartPending(status) ? "Restart to apply" : null}
    >
      {status ? (
        <div className={"mt-2 flex flex-col gap-6"}>
          <CheckboxFormRow
            label={"Doubled shader cache"}
            description={
              "Keeps 12 MB of compiled GPU pipelines between starts instead of 6 MB, so a level opens without compiling its shaders again."
            }
            isChecked={status.chosen.isShaderCacheDoubled}
            onChange={(isChecked: boolean) => onChange({ isShaderCacheDoubled: isChecked })}
          />

          <CheckboxFormRow
            label={"WebGPU developer features"}
            description={
              "Precise GPU timings for the renderer's passes, which the webview otherwise rounds to 65.5 µs."
            }
            isChecked={status.chosen.isWebgpuDeveloper}
            onChange={(isChecked: boolean) => onChange({ isWebgpuDeveloper: isChecked })}
          />
        </div>
      ) : (
        <Typography variant={"caption"}>Reading the webview options...</Typography>
      )}
    </DetailSection>
  );
}
