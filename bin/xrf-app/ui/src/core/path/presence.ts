import { exists } from "@tauri-apps/plugin-fs";

/**
 * Whether a path is there, reading a look the host refuses (out of scope, unreadable) as absent: for a caller that
 * only decides what to offer or show, where a refusal and a missing path call for the same answer.
 */
export async function isPathPresent(path: string): Promise<boolean> {
  return exists(path).catch(() => false);
}
