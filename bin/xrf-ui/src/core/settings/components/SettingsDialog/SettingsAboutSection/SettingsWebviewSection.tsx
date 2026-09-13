import { Box } from "@mui/material";
import { ReactElement } from "react";

import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { SettingsStat } from "@/core/settings/components/SettingsStat";
import { formatDuration } from "@/lib/format/duration";
import { formatBytes, formatBytesPair } from "@/lib/memory/format";
import { usePolledValue } from "@/lib/react";
import { Nullable } from "@/lib/types/general";

import { IWebviewStats, readWebviewStats } from "./webview-stats";

/** How often the window re-reads itself. Nothing here leaves the process, so it can afford the runtime section's rate. */
const POLL_INTERVAL: number = 1_000;

/**
 * What this window reports about itself, which a reload resets and the backend does not.
 */
export function SettingsWebviewSection(): ReactElement {
  const stats: Nullable<IWebviewStats> = usePolledValue(readWebviewStats, POLL_INTERVAL);
  const [heapUsed, heapTotal] = formatBytesPair(stats?.heap?.used ?? 0, stats?.heap?.total ?? 0);

  return (
    <SettingsSection
      data-testid={"settings-webview-section"}
      title={"Webview"}
      description={"What this window reports about itself, which a reload resets and the backend above does not."}
      fact={stats ? formatDuration(stats.age) : null}
    >
      <Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, marginTop: 1 }}>
        <SettingsStat
          label={"Script heap"}
          value={stats?.heap ? heapUsed : "not reported"}
          hint={stats?.heap ? `of ${heapTotal} allocated` : "this engine publishes none"}
        />
        <SettingsStat
          label={"Heap ceiling"}
          value={stats?.heap ? formatBytes(stats.heap.limit) : "—"}
          hint={"before the engine gives up"}
        />
        <SettingsStat label={"Elements"} value={(stats?.nodes ?? 0).toLocaleString()} hint={"in the document"} />
        <SettingsStat
          label={"Loaded in"}
          value={stats?.loadDuration ? formatDuration(stats.loadDuration) : "still loading"}
          hint={"to the load event"}
        />
      </Box>
    </SettingsSection>
  );
}
