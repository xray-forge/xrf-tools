/** A flush waiting for every record up to its number to be sent or dropped. */
export interface ILogFlushWaiter {
  sequence: number;
  resolve: () => void;
}
