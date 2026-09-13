import { Logger } from "@/lib/logging";
import { Nullable, Optional } from "@/lib/types/general";

/** One encoder for the process: constructing one per call would cost more than the measurement it serves. */
const ENCODER: TextEncoder = new TextEncoder();

/**
 * Weighs a value as the bytes it crosses as.
 *
 * @param value - Payload to weigh.
 * @returns Its size in bytes, or null when it cannot be serialized.
 */
export function weighIpcPayload(value: unknown): Nullable<number> {
  try {
    const json: Optional<string> = JSON.stringify(value);

    // An absent payload crosses as nothing rather than as unknown, which is a different statement.
    return json === undefined ? 0 : ENCODER.encode(json).length;
  } catch (error: unknown) {
    // A channel is a transport handle rather than data, and nothing here is worth failing a command over.
    Logger.warn("Cannot weigh IPC payload:", error);

    return null;
  }
}
