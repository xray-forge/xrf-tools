import { Nullable } from "@xrf/types";

import { EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { TOOL_CATALOG } from "@/ToolCatalog";

/**
 * Resolves a help entry's related tools to their descriptors.
 *
 * Help content names tools by id so a rename cannot strand a link; the descriptor supplies the
 * label, icon, and route. Planned tools are dropped rather than linked to a signpost.
 *
 * @param relatedTools - Tool ids named by a help entry.
 * @returns Descriptors of the ready tools among them.
 */
export function selectRelatedTools(relatedTools: ReadonlyArray<EToolId> = []): Array<IToolDescriptor> {
  return relatedTools
    .map((id: EToolId) => TOOL_CATALOG.findToolById(id))
    .filter(
      (tool: Nullable<IToolDescriptor>): tool is IToolDescriptor => tool !== null && tool.status === EToolStatus.READY
    );
}
