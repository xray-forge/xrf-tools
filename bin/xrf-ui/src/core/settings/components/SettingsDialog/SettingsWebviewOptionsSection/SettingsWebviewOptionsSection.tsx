import { Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import {
  EWebviewCollectionPace,
  WebviewCollectionPace,
  WebviewOptions,
  WebviewOptionsStatus,
} from "@/core/ipc/types/xrf-app";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { Logger, useLogger } from "@/lib/logging";
import { useMountEffect } from "@/lib/react";

import { isRestartPending, WEBVIEW_COLLECTION_PACE_OPTIONS } from "./SettingsWebviewOptionsSection.utils";

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

          <CheckboxFormRow
            label={"Vsync"}
            description={
              "Presents frames at the display's refresh. Off, frames are presented as soon as they are drawn, for measuring what a frame costs, and may tear."
            }
            isChecked={status.chosen.isVsync ?? true}
            onChange={(isChecked: boolean) => onChange({ isVsync: isChecked })}
          />

          <CheckboxFormRow
            label={"Frame rate limit"}
            description={
              "Keeps the page's frames to the display's refresh. With this and vsync off and the viewer's frame rate limit set to unlimited, the renderer runs as fast as it can."
            }
            isChecked={status.chosen.isFrameRateLimited ?? true}
            onChange={(isChecked: boolean) => onChange({ isFrameRateLimited: isChecked })}
          />

          <ChoiceFormRow
            label={"Garbage collection"}
            description={
              "How early the webview's JavaScript starts collecting garbage. Earlier collects more often, so the renderer's pause after each is shorter, for a little throughput."
            }
            options={WEBVIEW_COLLECTION_PACE_OPTIONS}
            value={status.chosen.collectionPace ?? EWebviewCollectionPace.DEFAULT}
            onChange={(collectionPace: WebviewCollectionPace) => onChange({ collectionPace })}
          />
        </div>
      ) : (
        <Typography variant={"caption"}>Reading the webview options...</Typography>
      )}
    </DetailSection>
  );
}
