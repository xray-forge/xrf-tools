import { describe, expect, it } from "@jest/globals";

import { Logger } from "@/lib/logging/Logger";

describe("module name token", () => {
  it("resolves to this file's own name", () => {
    // Asserted from inside a real module rather than by calling the substitution directly, because what matters is that
    // the token is gone by the time the code runs. A missed replacement throws on load and never reaches this line.
    expect(__MODULE_NAME__).toBe("module-name.test");
  });

  it("is what a logger built here tags its output with", () => {
    expect(new Logger(__MODULE_NAME__).prefix).toBe("module-name.test");
  });

  it("survives as a plain string, not an identifier the bundler can rename", () => {
    // The point of substituting at build time: `this.constructor.name` and `import.meta.url` both lose their meaning
    // once modules are minified into chunks, which is how a release build came to log `[U]`.
    expect(typeof __MODULE_NAME__).toBe("string");
  });
});
