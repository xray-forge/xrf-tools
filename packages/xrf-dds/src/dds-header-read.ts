import { Nullable } from "@xrf/types";

import { IDdsHeader } from "#/dds-header";
import { IDdsRefusal } from "#/dds-refusal";

/** What a header read came to: exactly one of the two is present. */
export interface IDdsHeaderRead {
  header: Nullable<IDdsHeader>;
  refusal: Nullable<IDdsRefusal>;
}
