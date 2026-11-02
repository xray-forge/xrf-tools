const swcJest = require("@swc/jest");

/**
 * Compiles a workspace member's TypeScript for jest with TypeScript's `experimentalDecorators` semantics, as the app's
 * bundler compiles it: wirestate's `@Observable()` fields and decorated flow methods behave as they do in the app only
 * under these, not under the standard decorators.
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
