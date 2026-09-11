const crypto = require("node:crypto");
const fs = require("node:fs");

const swcJest = require("@swc/jest");

const { replaceModuleName } = require("../build/module-name.ts");
const { applyObserver } = require("../build/observer.ts");

// Shared transforms and their locked dependencies all affect emitted code.
const SELF_HASH = crypto
  .createHash("sha1")
  .update(fs.readFileSync(__filename))
  .update(fs.readFileSync(require.resolve("../build/module-name.ts")))
  .update(fs.readFileSync(require.resolve("../build/observer.ts")))
  .update(fs.readFileSync(require.resolve("../../pnpm-lock.yaml")))
  .digest("hex")
  .slice(0, 12);

/**
 * Compile with swc, after wrapping components in `observer()` with babel.
 *
 * Two transforms in the order `vite.config.ts` applies them: the observer plugin runs `pre` on source
 * that still has its JSX, then the typescript compile happens. Splitting the work this way is not a
 * preference - the observer plugin exists only for babel, while decorators need swc, which implements
 * typescript's `experimentalDecorators` semantics rather than babel's approximation of them. Babel's
 * legacy decorator transform produced services whose `@Observable()` fields were not reactive.
 */
function createSwcOptions(isTsx) {
  return {
    jsc: {
      parser: {
        syntax: "typescript",
        tsx: isTsx,
        decorators: true,
      },
      target: "es2022",
      transform: {
        decoratorMetadata: true,
        legacyDecorator: true,
        react: {
          development: true,
          runtime: "automatic",
        },
      },
    },
    module: { type: "commonjs" },
    sourceMaps: true,
  };
}

const tsxTransformer = swcJest.createTransformer(createSwcOptions(true));
const tsTransformer = swcJest.createTransformer(createSwcOptions(false));

module.exports = {
  process(source, filename, options) {
    const transformer = filename.endsWith(".tsx") ? tsxTransformer : tsTransformer;

    // Runs before the observer pass for the same reason it is `enforce: "pre"` in vite: the token is source text, and
    // every later stage should see the name rather than the placeholder. Unlike the observer step this applies to
    // `.ts` too, which is where the services that use it live.
    const named = replaceModuleName(source, filename);

    return transformer.process(applyObserver(named, filename), filename, options);
  },
  getCacheKey(source, filename, options) {
    const transformer = filename.endsWith(".tsx") ? tsxTransformer : tsTransformer;

    // SWC's key omits the shared observer transform and its dependencies.
    return `${transformer.getCacheKey(source, filename, options)}-${SELF_HASH}`;
  },
};
