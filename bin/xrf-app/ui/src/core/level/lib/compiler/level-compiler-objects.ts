import type { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";
import { IStoredValueCodec, jsonCodec } from "@/core/storage";

/**
 * How the level compiler remembers its objects roots ({@link ELevelCompilerField.OBJECTS}): a JSON list, nothing kept
 * for none. The compiler reads the field through it and a hand-over writes it, so both keep the same form.
 */
export const LEVEL_COMPILER_OBJECTS_CODEC: IStoredValueCodec<Array<string>> = jsonCodec(
  (parsed: unknown): Array<string> =>
    Array.isArray(parsed) ? parsed.filter((root: unknown): root is string => typeof root === "string") : [],
  (roots: Array<string>) => (roots.length > 0 ? JSON.stringify(roots) : null)
);
