import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { EApplicationId, EApplicationStatus, IApplicationDescriptor } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

/**
 * Resolves a help entry's related tools to their descriptors.
 *
 * Help content names applications by id so a rename cannot strand a link; the descriptor supplies the
 * label, icon, and route. Planned applications are dropped rather than linked to a signpost.
 *
 * @param relatedTools - Application ids named by a help entry.
 * @returns Descriptors of the ready applications among them.
 */
export function selectRelatedApplications(
  relatedTools: ReadonlyArray<EApplicationId> = []
): Array<IApplicationDescriptor> {
  return relatedTools
    .map((id: EApplicationId) => APPLICATION_CATALOG.findApplicationById(id))
    .filter(
      (application: Nullable<IApplicationDescriptor>): application is IApplicationDescriptor =>
        application !== null && application.status === EApplicationStatus.READY
    );
}
