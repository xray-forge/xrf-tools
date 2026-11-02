import { describe, expect, it } from "@jest/globals";

import { transformError } from "#/transform-error";
import { XrfApplicationError } from "#/xrf-application-error";

describe("transformError", () => {
  it("passes an error through as it was thrown", () => {
    const error: XrfApplicationError = new XrfApplicationError("refused");

    expect(transformError(error)).toBe(error);
  });

  it("wraps anything else in an error carrying its text", () => {
    const transformed: Error = transformError("plain text rejection");

    expect(transformed).toBeInstanceOf(Error);
    expect(transformed.message).toBe("plain text rejection");
  });
});
