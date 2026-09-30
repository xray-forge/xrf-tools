import { Nullable } from "@xrf/types";

import { IDdsFile } from "#/dds-file";
import { IDdsRefusal } from "#/dds-refusal";

/**
 * What a read came to: exactly one of the two is present.
 */
export interface IDdsRead {
  file: Nullable<IDdsFile>;
  refusal: Nullable<IDdsRefusal>;
}
