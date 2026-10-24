const swcJest = require("@swc/jest");

/**
 * Compiles the package for jest with TypeScript's `experimentalDecorators` semantics, which the session layer's
 * wirestate services need for their `@Observable()` fields to be reactive, as the app compiles them.
 */
module.exports = swcJest.createTransformer({
  jsc: {
    parser: {
      syntax: "typescript",
      tsx: false,
      decorators: true,
    },
    target: "es2022",
    transform: {
      decoratorMetadata: true,
      legacyDecorator: true,
    },
  },
  module: { type: "commonjs" },
  sourceMaps: true,
});
