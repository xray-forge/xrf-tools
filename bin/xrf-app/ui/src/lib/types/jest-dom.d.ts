// Registers jest-dom's matchers on the `expect` exported by `@jest/globals`. The runtime side is
// wired in `cli/test/jest_setup.ts`; without this import tsc does not see `toBeInTheDocument`.
import "@testing-library/jest-dom/jest-globals";
