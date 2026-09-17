import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveContent, useLastContent } from "@/core/archive/lib";
import {
  ArchiveDescribeRefusal,
  ArchiveFileDescription,
  EArchiveDescribeRefusal,
  EArchiveFormatDescription,
} from "@/core/ipc/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { AsyncState } from "@/lib/async-state";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

import { ArchivePreviewError } from "../ArchivePreviewError";
import { ArchiveAnmDescriptionView } from "./ArchiveAnmDescriptionView";
import { ArchiveChunksDescriptionView } from "./ArchiveChunksDescriptionView";
import { ArchiveDetailDescriptionView } from "./ArchiveDetailDescriptionView";
import { ArchiveDetailLibraryDescriptionView } from "./ArchiveDetailLibraryDescriptionView";
import { ArchiveEfdView } from "./ArchiveEfdView";
import { ArchiveGameMtlView } from "./ArchiveGameMtlView";
import { ArchiveLevelDescriptionView } from "./ArchiveLevelDescriptionView";
import { ArchiveLevelEnvModView } from "./ArchiveLevelEnvModView";
import { ArchiveLevelFogVolView } from "./ArchiveLevelFogVolView";
import { ArchiveLevelGameView } from "./ArchiveLevelGameView";
import { ArchiveLevelGeomView } from "./ArchiveLevelGeomView";
import { ArchiveLevelLightsView } from "./ArchiveLevelLightsView";
import { ArchiveLevelHomView, ArchiveLevelSomView } from "./ArchiveLevelOcclusionView";
import { ArchiveLevelAiView, ArchiveLevelCollisionView } from "./ArchiveLevelPartView";
import { ArchiveLevelPsStaticView } from "./ArchiveLevelPsStaticView";
import { ArchiveLevelSndStaticView } from "./ArchiveLevelSndStaticView";
import { ArchiveLevelSpawnView } from "./ArchiveLevelSpawnView";
import { ArchiveLevelWallmarksView } from "./ArchiveLevelWallmarksView";
import { ArchiveLightAnimView } from "./ArchiveLightAnimView";
import { ArchiveOmfDescriptionView } from "./ArchiveOmfDescriptionView";
import { ArchiveParticlesDescriptionView } from "./ArchiveParticlesDescriptionView";
import { ArchivePpeDescriptionView } from "./ArchivePpeDescriptionView";
import { ArchiveShaderCompilerView } from "./ArchiveShaderCompilerView";
import { ArchiveShadersDescriptionView } from "./ArchiveShadersDescriptionView";
import { ArchiveSoundEnvironmentView } from "./ArchiveSoundEnvironmentView";
import { ArchiveSpawnDescriptionView } from "./ArchiveSpawnDescriptionView";
import { ArchiveThmDescriptionView } from "./ArchiveThmDescriptionView";

/**
 * What the backend can say about a binary entry the viewer cannot draw.
 */
export function ArchiveDescriptionPreview({
  "data-testid": dataTestId = "archive-description-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

  // The previous description stays on screen while the next one is read, rather than the panel blanking between
  // clicks - the same courtesy the texture and sound previews extend.
  const described: Nullable<TArchiveContent & { kind: "description" }> = useLastContent(
    content.value?.kind === "description" ? content.value : null,
    content.isLoading
  );

  const onGetRefusalDescription = useCallback((reason: ArchiveDescribeRefusal): string => {
    switch (reason.kind) {
      case EArchiveDescribeRefusal.NO_DESCRIBER:
        return reason.extension
          ? `Nothing reads .${reason.extension} files yet. Their metadata is still available in Details.`
          : "Nothing reads files without an extension yet. Their metadata is still available in Details.";
      case EArchiveDescribeRefusal.TOO_LARGE:
        return (
          `This file is ${formatBytes(reason.size)}, past the ${formatBytes(reason.maximum)} limit for reading a ` +
          "file whole to describe it. Its metadata is still available in Details."
        );
      default:
        return assertExhaustive(reason);
    }
  }, []);

  if (content.error) {
    return (
      <ArchivePreviewError
        data-testid={dataTestId}
        id={id}
        className={className}
        error={content.error}
        onRetry={archivesService.retrySelectedFile}
      />
    );
  }

  if (!described) {
    return content.isLoading ? (
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    ) : (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"Nothing to show"}
        description={"The selected file did not return a description."}
      />
    );
  }

  const description: ArchiveFileDescription = described.description;

  switch (description.format.kind) {
    case EArchiveFormatDescription.ANM:
      return (
        <ArchiveAnmDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.CHUNKS:
      return (
        <ArchiveChunksDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.DETAIL:
      return (
        <ArchiveDetailDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.DETAIL_LIBRARY:
      return (
        <ArchiveDetailLibraryDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.EFD:
      return (
        <ArchiveEfdView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.GAME_MTL:
      return (
        <ArchiveGameMtlView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL:
      return (
        <ArchiveLevelDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.LEVEL_AI:
      return (
        <ArchiveLevelAiView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_COLLISION:
      return (
        <ArchiveLevelCollisionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_ENV_MOD:
      return (
        <ArchiveLevelEnvModView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_FOG_VOL:
      return (
        <ArchiveLevelFogVolView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.LEVEL_GAME:
      return (
        <ArchiveLevelGameView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_GEOM:
      return (
        <ArchiveLevelGeomView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_HOM:
      return (
        <ArchiveLevelHomView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_LIGHTS:
      return (
        <ArchiveLevelLightsView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_PS_STATIC:
      return (
        <ArchiveLevelPsStaticView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_SND_STATIC:
      return (
        <ArchiveLevelSndStaticView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.LEVEL_SOM:
      return (
        <ArchiveLevelSomView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_SPAWN:
      return (
        <ArchiveLevelSpawnView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.LEVEL_WALLMARKS:
      return (
        <ArchiveLevelWallmarksView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.LIGHT_ANIM:
      return (
        <ArchiveLightAnimView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.SHADER_COMPILER:
      return (
        <ArchiveShaderCompilerView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.SOUND_ENVIRONMENT:
      return (
        <ArchiveSoundEnvironmentView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.OMF:
      return (
        <ArchiveOmfDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.PARTICLES:
      return (
        <ArchiveParticlesDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.PPE:
      return (
        <ArchivePpeDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.SHADERS:
      return (
        <ArchiveShadersDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.SPAWN:
      return (
        <ArchiveSpawnDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
        />
      );

    case EArchiveFormatDescription.THM:
      return (
        <ArchiveThmDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case EArchiveFormatDescription.UNSUPPORTED:
      return (
        <EmptyState
          data-testid={dataTestId}
          id={id}
          className={className}
          title={"No description yet"}
          description={onGetRefusalDescription(description.format.reason)}
        />
      );

    default:
      return assertExhaustive(description.format);
  }
}
