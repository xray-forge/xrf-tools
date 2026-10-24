export const PREFIX_TIMESTAMP_TAG: object = {
  toString(): string {
    return new Date().toLocaleTimeString("en-GB", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      fractionalSecondDigits: 3,
    });
  },
};
