/**
 * Source repository this application is built from.
 * todo: Use package json on compile time as better idea.
 */
export const REPOSITORY_URL: string = "https://github.com/xray-forge/xrf-tools";

/**
 * Address of one workflow run, which build details link to so a binary can be traced to what produced it.
 */
export function getWorkflowRunUrl(runId: string): string {
  return `${REPOSITORY_URL}/actions/runs/${runId}`;
}
