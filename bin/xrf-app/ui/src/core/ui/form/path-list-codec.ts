import { IStoredValueCodec, jsonCodec } from "@/core/storage";

/**
 * How a list of paths a form remembers is kept: a JSON list, nothing kept for none. A tool reads its list field through
 * it and a hand-over writes the next tool's, so both keep the same form.
 */
export const PATH_LIST_CODEC: IStoredValueCodec<Array<string>> = jsonCodec(
  (parsed: unknown): Array<string> =>
    Array.isArray(parsed) ? parsed.filter((path: unknown): path is string => typeof path === "string") : [],
  (paths: Array<string>) => (paths.length > 0 ? JSON.stringify(paths) : null)
);
