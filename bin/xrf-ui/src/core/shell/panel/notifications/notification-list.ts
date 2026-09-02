import { ENotificationSeverity, INotification } from "@/core/notifications/lib";
import { Nullable } from "@/lib/types/general";

/**
 * Severities that colour their own title and show a line of `details` before anyone asks for it.
 *
 * A failure's first line is usually the reason it failed, which is the one line worth spending on it
 * unprompted. Every other severity keeps its body collapsed until it is opened.
 */
const ATTENTION_SEVERITIES: ReadonlySet<ENotificationSeverity> = new Set([
  ENotificationSeverity.ERROR,
  ENotificationSeverity.WARNING,
]);

/** One rendered row: a record, plus the identical records directly beneath it that it stands for. */
export interface INotificationEntry {
  notification: INotification;
  /** How many records this row represents, never below one. */
  repeatCount: number;
}

/**
 * Whether a severity is urgent enough to be coloured and previewed rather than left to the pile.
 *
 * @param severity - Severity of the recorded outcome.
 * @returns Whether the row states its severity in the title as well as the icon.
 */
export function isAttentionSeverity(severity: ENotificationSeverity): boolean {
  return ATTENTION_SEVERITIES.has(severity);
}

/**
 * Two records are one event repeated when everything a row would show about them agrees.
 */
function isSameEvent(one: INotification, other: INotification): boolean {
  return (
    one.severity === other.severity &&
    one.source === other.source &&
    one.title === other.title &&
    one.details === other.details
  );
}

/**
 * Turn the stored log into the rows the panel draws.
 *
 * A run of the same event becomes one counted row, which is what keeps a loop that traces every
 * iteration from burying the outcome that followed it.
 *
 * @param notifications - Records newest first, as the service keeps them.
 * @returns Rows newest first.
 */
export function toNotificationEntries(notifications: Array<INotification>): Array<INotificationEntry> {
  const entries: Array<INotificationEntry> = [];

  for (const notification of notifications) {
    const previous: Nullable<INotificationEntry> = entries.at(-1) ?? null;

    if (previous && isSameEvent(previous.notification, notification)) {
      previous.repeatCount += 1;
      continue;
    }

    entries.push({ notification, repeatCount: 1 });
  }

  return entries;
}
