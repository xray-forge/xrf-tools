import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { ArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { EPathEntryKind } from "@xrf/path";
import { Nullable } from "@xrf/types";

import { mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { IArchiveEntry, TArchiveContent } from "@/core/archive/lib";
import { mockCommandAnswer } from "@/core/operations/fixtures/command-operation.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

/** The archives explorer's services over one container, as its tool binds them. */
export interface IArchivesExplorerMock {
  container: Container;
  archivesService: ArchivesService;
  previewService: ArchivePreviewService;
}

/**
 * Builds the explorer's services, holding `subject` open where one is given.
 *
 * @param subject - What the explorer holds open, or none.
 * @returns The services and their container.
 */
export function mockArchivesExplorer(subject: Nullable<SessionSnapshot<ArchiveSubject>> = null): IArchivesExplorerMock {
  const { container, service: previewService } = mockInjectedService(ArchivePreviewService, [ArchivesService]);
  const archivesService: ArchivesService = container.get(ArchivesService);

  if (subject) {
    mockHeldSessionRoot(archivesService.subject, subject);
  }

  return { archivesService, container, previewService };
}

/**
 * Shows one file of a held volume set as selecting and reading it would, for a test whose setup cannot wait for the
 * read.
 *
 * @param explorer - The explorer's services.
 * @param entry - The file shown.
 * @param content - What its read answered, or null for one that answered nothing.
 */
export function mockShownArchiveFile(
  explorer: IArchivesExplorerMock,
  entry: IArchiveEntry,
  content: Nullable<TArchiveContent>
): void {
  const { archivesService, previewService } = explorer;
  const sessionId: string = "fixture-session";

  if (archivesService.subject.sessionId === null) {
    mockHeldSessionRoot(archivesService.subject, mockSessionSnapshot(mockArchivesVolumes([]), sessionId));
  }

  mockCommandAnswer(previewService["read"], AsyncState.ready(content));

  runInAction(() => {
    previewService["readFor"] = archivesService.subject.sessionId;
    previewService["view"] = {
      selection: { entry, kind: EPathEntryKind.FILE },
      sessionId: archivesService.subject.sessionId,
    };
  });
}
