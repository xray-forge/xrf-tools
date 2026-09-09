/**
 * Source repository this application is built from.
 */
export const REPOSITORY_URL: string = __REPOSITORY_URL__.replace(/^git\+/, "").replace(/(?:\.git)?\/?$/, "");

/**
 * Address of the exact commit used to build the application.
 */
export function getCommitUrl(commit: string): string {
  return `${REPOSITORY_URL}/commit/${commit}`;
}

/**
 * Address of one workflow run, which build details link to so a binary can be traced to what produced it.
 */
export function getWorkflowRunUrl(runId: string): string {
  return `${REPOSITORY_URL}/actions/runs/${runId}`;
}
