import { Channel } from "@tauri-apps/api/core";
import { JobProgress } from "@xrf/ipc/types/xrf-job";

/** The job a capture of the viewport runs as: its id, and where its progress is told. */
export interface ILevelRenderJob {
  id: string;
  progress: Channel<JobProgress>;
}
