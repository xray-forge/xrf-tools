const swcJest = require("@swc/jest");

/**
 * Compiles a workspace member's TypeScript and TSX for jest, the way its bundler would: JSX through React's automatic
 * runtime, and a `.ts` file parsed without JSX, so its generic arrows and angle-bracket casts keep their meaning.
 *
 * @param {boolean} isTsx - Whether the file may hold JSX.
 * @returns {import("@swc/core").Options} The compile options.
 */
function createSwcOptions(isTsx) {
  return {
    jsc: {
      parser: {
        syntax: "typescript",
        tsx: isTsx,
      },
      target: "es2022",
      transform: {
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
    return (filename.endsWith(".tsx") ? tsxTransformer : tsTransformer).process(source, filename, options);
  },
  getCacheKey(source, filename, options) {
    return (filename.endsWith(".tsx") ? tsxTransformer : tsTransformer).getCacheKey(source, filename, options);
  },
};
