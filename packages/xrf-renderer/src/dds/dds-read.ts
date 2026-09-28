import { Nullable } from "@xrf/types";

import { IDdsFile } from "#/dds/dds-file";
import { IDdsRefusal } from "#/dds/dds-refusal";

/**
 * What a read came to: exactly one of the two is present.
 */
export interface IDdsRead {
  file: Nullable<IDdsFile>;
  refusal: Nullable<IDdsRefusal>;
}
