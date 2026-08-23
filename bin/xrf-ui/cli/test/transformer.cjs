const crypto = require("node:crypto");
const fs = require("node:fs");

const babel = require("@babel/core");
const swcJest = require("@swc/jest");
const observingComponents = require("babel-plugin-observing-components");

const { replaceModuleName } = require("../build/module-name.ts");

// The package default-exports a factory returning a `[plugin, options]` tuple, so it has to be called
// rather than referenced by name. This is the identical call `mobx-react-observer/vite-plugin` makes.
const createObserverPlugin = observingComponents.default ?? observingComponents;

// Covers the shared module-name substitution too: it is as much a part of the output as this file is, and a cache keyed
// on only one of them serves code compiled by the other's previous version.
const SELF_HASH = crypto
  .createHash("sha1")
  .update(fs.readFileSync(__filename))
  .update(fs.readFileSync(require.resolve("../build/module-name.ts")))
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

function applyObserver(source, filename) {
  // Mirrors the `.tsx` guard in `vite.config.ts`: only component files carry JSX worth wrapping.
  if (!filename.endsWith(".tsx") || filename.includes("node_modules")) {
    return source;
  }

  const result = babel.transformSync(source, {
    babelrc: false,
    configFile: false,
    filename,
    // `decorators-legacy` matches the `legacyDecorator` swc runs with. Babel only has to parse here,
    // not transform, but a `.tsx` carrying a decorated class fails at the parse without it.
    parserOpts: { plugins: ["jsx", "typescript", "decorators-legacy"] },
    plugins: [createObserverPlugin({ importPath: "mobx-react-observer" })],
  });

  return result?.code ?? source;
}

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

    // Salted with this file's own contents. swc's key covers the source and its own options but knows
    // nothing about the observer step, so without this an edit here silently reuses output compiled
    // by the previous version - which is exactly how a broken transform passes its own tests.
    return `${transformer.getCacheKey(source, filename, options)}-${SELF_HASH}`;
  },
};
