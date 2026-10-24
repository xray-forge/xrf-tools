import { readdirSync } from "fs";
import { resolve } from "path";

import { describe, expect, it, jest } from "@jest/globals";

import { ARCHIVES_JOB_PACK } from "@/core/ipc/types/xrf-app-archives";

import { findJobKind, JOB_KINDS } from "./job-kinds";

/** Where the generated type modules are, one per crate, each app crate declaring its own domain's job kinds. */
const TYPES_DIRECTORY: string = resolve(__dirname, "../../ipc/types");

/**
 * Every job kind the backend declares, as the generated `<DOMAIN>_JOB_<OPERATION>` constants of the app crates' modules
 * carry them: read from every such module there is, so a plugin added later is held to its descriptors too.
 */
const DECLARED_KINDS: Array<string> = readdirSync(TYPES_DIRECTORY)
  .filter((file) => file.startsWith("xrf-app-") && file.endsWith(".ts"))
  .flatMap((file) => Object.entries(jest.requireActual<Record<string, unknown>>(resolve(TYPES_DIRECTORY, file))))
  .filter(([name, value]) => /^[A-Z][A-Z_]*_JOB_[A-Z_]+$/.test(name) && typeof value === "string")
  .map(([, value]) => value as string);

describe("JOB_KINDS", () => {
  it("reads the generated job kinds", () => {
    expect(DECLARED_KINDS).toContain(ARCHIVES_JOB_PACK);
  });

  it("describes every job kind the backend declares, under its own kind", () => {
    for (const kind of DECLARED_KINDS) {
      expect(findJobKind(kind)?.kind).toBe(kind);
    }
  });

  it("describes no kind the backend does not declare", () => {
    expect(Object.keys(JOB_KINDS).sort()).toEqual([...DECLARED_KINDS].sort());
  });
});
