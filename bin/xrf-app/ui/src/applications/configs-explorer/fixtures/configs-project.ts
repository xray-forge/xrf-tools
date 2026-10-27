import { mockInvoke } from "@xrf/ipc/testing";
import { ConfigsProjectDescriptor } from "@xrf/ipc/types/xrf-app-configs";

import { ConfigsProjectService } from "@/applications/configs-explorer/services/project";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";

/**
 * Puts an open configs project on a service the way a reload restores one, answering only that restore, so the
 * responses a test configured stay in place.
 *
 * @param service - Service to hold the project.
 * @param sessionId - Identity every read of the project is addressed by.
 * @param project - What the project is, for the tests that read it.
 * @returns Resolves once the project is held.
 */
export async function mockRestoredConfigsProject(
  service: ConfigsProjectService,
  sessionId: string = "session-1",
  project: Partial<ConfigsProjectDescriptor> = {}
): Promise<void> {
  mockInvoke.mockResolvedValueOnce(mockSessionSnapshot(project as ConfigsProjectDescriptor, sessionId));

  await service.onProvision();
}
