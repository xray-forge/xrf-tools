import { EDdsRefusalReason } from "#/dds/dds-refusal-reason";

/** Why one file was refused, in the terms whoever reports it needs. */
export interface IDdsRefusal {
  reason: EDdsRefusalReason;
  /** What was actually in the header, so a report can name the layout rather than only its category. */
  detail: string;
}
