import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Alert, Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  EArchiveSubject,
  getArchiveVolumeOf,
  getSubjectRoot,
  getSubjectShadowedCount,
  getSubjectSize,
  getSubjectSourceCount,
} from "@/core/archive";
import { describeAssetContainer } from "@/core/assets/lib";
import { ArchiveSubject } from "@/core/bindings/types/xrf-app";
import { ArchiveDescriptor } from "@/core/bindings/types/xrf-archive";
import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation, IEditorLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

import { ARCHIVE_EDITOR_PANELS } from "./archive-panels";
import { ArchivesFilePreview } from "./preview";
import { ArchivesMenu } from "./tree";

export function ArchivesEditor(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const [isClosing, setClosing] = useState<boolean>(false);
  const [closeError, setCloseError] = useState<Nullable<string>>(null);
  const [isCollisionNoticeDismissed, setCollisionNoticeDismissed] = useState<boolean>(false);
  const [isShadowNoticeDismissed, setShadowNoticeDismissed] = useState<boolean>(false);

  const subject: Nullable<ArchiveSubject> = archivesService.subject.value;
  const collisions: Array<XrayPathCollision> = archivesService.collisions.value ?? [];

  const isWorld: boolean = subject?.kind === EArchiveSubject.WORLD;
  const shadowedCount: number = getSubjectShadowedCount(subject);

  // A volume set names the volume its name table points into; a world names where the winning copy actually sits.
  const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(
    subject?.kind === EArchiveSubject.VOLUMES ? subject.project : null,
    archivesService.selectedDescriptor
  );
  const container: Nullable<string> = archivesService.selectedWorldEntry
    ? describeAssetContainer(archivesService.selectedWorldEntry.container)
    : null;
  const source: Nullable<string> = container ?? volume?.path ?? null;

  const root: string = getSubjectRoot(subject);
  const location: Nullable<IEditorLocation> = source
    ? { entry: archivesService.selectedEntry?.name, path: source }
    : root
      ? { path: root }
      : null;

  // The run rather than the service's own flag: an extraction survives the window being reloaded, so returning here
  // finds it again instead of showing an idle tree over files it is still writing.
  const job: Nullable<IJobState> = archivesService.job;

  // Extraction writes to disk outside the archive. Walking away mid-write left it running against a
  // screen nobody could see, and the only signal it was happening was one button in the content area.
  const isExtracting: boolean = archivesService.operation.isLoading;
  const isBusy: boolean = isClosing || isExtracting;
  const isCollisionNoticeShown: boolean = collisions.length > 0 && !isCollisionNoticeDismissed;
  const isShadowNoticeShown: boolean = shadowedCount > 0 && !isShadowNoticeDismissed;

  const onCancelExtraction = useCallback(() => archivesService.cancelExtraction(), [archivesService]);

  const onClose = useCallback(async (): Promise<void> => {
    setClosing(true);
    setCloseError(null);

    try {
      await archivesService.closeSubject();
    } catch (error: unknown) {
      setCloseError(error instanceof Error ? error.message : String(error));
    } finally {
      setClosing(false);
    }
  }, [archivesService]);

  useEditorPanels(
    () => [
      {
        icon: <FolderOpenIcon />,
        id: "archives",
        isOpenByDefault: true,
        label: "Archives",
        render: () => <ArchivesMenu />,
        side: "left",
      },
      ...ARCHIVE_EDITOR_PANELS,
    ],
    []
  );

  useEditorBusy(isBusy || Boolean(job));

  useEditorStatus([
    `${getSubjectSourceCount(subject)} ${isWorld ? "sources" : "archives"}`,
    `${archivesService.entries.length} files`,
    formatBytes(getSubjectSize(subject)),
    ...(isWorld ? [`${shadowedCount} overridden`] : []),
  ]);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={location ? <EditorToolbarLocation location={location} /> : null}
          onBack={() => void onClose()}
        />
      }
      banner={
        job || closeError || isCollisionNoticeShown || isShadowNoticeShown ? (
          <>
            {job ? (
              <Box sx={{ paddingX: 2, paddingY: 1 }}>
                <JobProgressView job={job} onCancel={onCancelExtraction} />
              </Box>
            ) : null}

            {closeError ? (
              <Alert severity={"error"} onClose={() => setCloseError(null)}>
                Could not close archives: {closeError}
              </Alert>
            ) : null}

            {isShadowNoticeShown ? (
              <Alert
                severity={"info"}
                closeText={"Dismiss overridden files notice"}
                onClose={() => setShadowNoticeDismissed(true)}
              >
                {shadowedCount} file(s) here exist in more than one source. The tree shows the copy the engine would
                load; the rest are listed under Origin in File details.
              </Alert>
            ) : null}

            {isCollisionNoticeShown ? (
              <Alert
                severity={"warning"}
                closeText={"Dismiss unreachable files notice"}
                onClose={() => setCollisionNoticeDismissed(true)}
              >
                {collisions.length} file(s) here cannot be reached - another entry claims their engine path. See the
                Unreachable files panel.
              </Alert>
            ) : null}
          </>
        ) : null
      }
    >
      <ArchivesFilePreview />
    </EditorLayout>
  );
}
