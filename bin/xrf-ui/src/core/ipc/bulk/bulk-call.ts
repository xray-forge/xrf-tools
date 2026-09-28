/**
 * One call of a bulk route, as data: what the generated `<plugin>-bulk.ts` wrappers answer, so the page can fetch it
 * or hand it to another thread to fetch.
 */
export interface IBulkCall {
  /** `<plugin>/<route>`, such as `levels/read_sector`. */
  route: string;
  /** The route's arguments, sent as its JSON body. */
  args: Record<string, unknown>;
}
