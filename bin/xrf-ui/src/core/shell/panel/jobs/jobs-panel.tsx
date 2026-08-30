import { default as PlaylistPlayIcon } from "@mui/icons-material/PlaylistPlay";

import { IEditorPanel } from "@/core/shell/panel/context";
import { JobsPanel } from "@/core/shell/panel/jobs/JobsPanel";

/**
 * The running-work listing, owned by the frame and registered only in dev mode.
 * Gated because a person using the tools has no need of it.
 */
export const JOBS_PANEL: IEditorPanel = {
  icon: <PlaylistPlayIcon />,
  id: "jobs",
  isOpenByDefault: false,
  label: "Jobs",
  render: () => <JobsPanel />,
  side: "right",
};
