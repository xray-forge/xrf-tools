import { PlaylistPlayIcon } from "@xrf/ui/icons/playlist-play";

import { JobsPanel } from "@/core/shell/panels/jobs/JobsPanel";
import { IShellPanel } from "@/core/shell/panels/shell-panel";

/**
 * The running-work listing, owned by the frame and registered only in dev mode.
 * Gated because a person using the tools has no need of it.
 */
export const JOBS_PANEL: IShellPanel = {
  icon: <PlaylistPlayIcon />,
  id: "jobs",
  isOpenByDefault: false,
  label: "Jobs",
  render: () => <JobsPanel />,
  side: "right",
};
