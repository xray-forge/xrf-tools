import "@testing-library/jest-dom";

import { afterEach, jest } from "@jest/globals";
import { cleanup } from "@testing-library/react";

// jsdom has no layout or native scrolling.
Object.defineProperty(HTMLElement.prototype, "scrollIntoView", { configurable: true, value: jest.fn() });

afterEach(async () => {
  cleanup();

  await new Promise((resolve) => setTimeout(resolve, 0));
});
