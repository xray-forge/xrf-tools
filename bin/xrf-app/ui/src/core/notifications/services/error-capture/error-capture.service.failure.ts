import { Nullable } from "@xrf/types";

/** One failure nothing else reports, as the listener that caught it describes it. */
export interface ICapturedFailure {
  /** What failed, opening its log line: an uncaught error and where it was thrown, or an unhandled rejection. */
  what: string;
  /** The notification's details: where an uncaught error was thrown, or that a rejection went unhandled. */
  details: string;
  /** Its message: the notification's title, and the log line's value when it carries no `Error`. */
  message: string;
  /** The `Error` thrown, written to the log as its stack; null when what was thrown is no `Error`. */
  cause: Nullable<Error>;
}
