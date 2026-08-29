import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { Box } from "@mui/material";
import { ReactElement, ReactNode, useCallback, useMemo, useState } from "react";
import { Texture } from "three";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { IEditorPanel, useEditorPanels } from "@/core/shell/panel/context";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import {
  IVisualPreviewViewportProps,
  VisualPreviewEmpty,
  VisualPreviewMotionViewport,
  VisualPreviewToolbar,
} from "@/core/visuals/components/preview";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, IVisualPreviewViewOptions } from "@/core/visuals/components/scene";
import { countVisualTriangles, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualPreviewLayoutProps extends BaseComponentProps {
  /** The model on screen, or null while nothing is open. */
  model?: Nullable<IVisualModelViews>;
  /** Shown in the toolbar beside the view toggles, usually where the model came from. */
  subtitle?: string;
  /** Published as a left panel when given. Opening a single visual has nothing to browse. */
  tree?: ReactNode;
  /** Data panels the owning application contributes to the right stripe. */
  panels?: Array<IEditorPanel>;
  /** Loaded textures by submesh index, passed straight through to the viewport. */
  textures?: ReadonlyMap<number, Texture>;
  /** Joint to mark in the viewport, named elsewhere - the bones panel - and resolved to a position by its owner. */
  highlightedJoint?: Nullable<[number, number, number]>;
  /** Bones the viewport collapses, by index, as the engine collapses an addon that is not attached. */
  hiddenBones?: ReadonlySet<number>;
  /**
   * Draws the viewport, for a surface that poses the model from something other than a single picked motion.
   *
   * A render prop rather than a node, because the model, the view toggles, the detail level and the camera reset token
   * are this layout's to own: a caller passing a finished element would have to be handed all four back.
   */
  renderViewport?: (props: IVisualPreviewViewportProps) => ReactNode;
  /** Drawn under the viewport, at whatever height it asks for. */
  footer?: ReactNode;
  /** Whether a model is on its way, reported over the viewport rather than by replacing the screen. */
  isLoading?: boolean;
  /** Why the last open failed, shown in place of a model rather than dismissing the session. */
  error?: string;
  /** Reads the failed open's source again. Absent while an application cannot repeat its last attempt. */
  onRetry?: () => void;
  /** Reopens the picker. Absent while an application has no way to choose a different visual. */
  onOpen?: () => void;
  /** Promotes a single-model session to a browsed one. Absent while already browsing. */
  onBrowse?: () => void;
}

/**
 * The shared preview chrome: toolbar, viewport, panel stripe and animation bar.
 *
 * Data comes in as props rather than being read here, so this stays usable by an application that has a
 * backing service and by one that does not. Playback is the exception: the bar and the pose read
 * `VisualMotionService` themselves, so an application mounting this has to bind it.
 */
export function VisualPreviewLayout({
  "data-testid": dataTestId = "visual-preview-layout",
  id = "visual-preview-layout",
  className,
  model = null,
  subtitle,
  tree,
  panels,
  textures,
  highlightedJoint = null,
  hiddenBones,
  renderViewport,
  footer,
  isLoading = false,
  error,
  onRetry,
  onOpen,
  onBrowse,
}: IVisualPreviewLayoutProps): ReactElement {
  const [options, setOptions] = useState<IVisualPreviewViewOptions>(DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS);
  const [cameraResetToken, setCameraResetToken] = useState(0);
  const [detail, setDetail] = useState(0);

  const onResetCamera = useCallback(() => setCameraResetToken((it) => it + 1), []);

  /**
   * Detail is a fraction of each submesh's collapse chain, so it needs no clamping and survives a model change
   * meaningfully: stepping through a tree at half detail stays at half detail whether the next model carries four
   * collapse steps or nine hundred.
   */
  const triangleCount: number = model ? countVisualTriangles(model, detail) : 0;
  const hasDetailLevels: boolean = (model?.levelCount ?? 1) > 1;
  const hasSkeleton: boolean = Boolean(model?.skeleton);

  useEditorPanels(() => {
    const stripe: Array<IEditorPanel> = panels ? [...panels] : [];

    return tree
      ? [
          {
            icon: <AccountTreeIcon />,
            id: "project",
            isOpenByDefault: true,
            label: "Project",
            render: () => tree,
            side: "left",
          },
          ...stripe,
        ]
      : stripe;
  }, [tree, panels]);

  const status: Array<string> = useMemo(() => {
    if (isLoading) {
      return ["Loading visual"];
    }

    return model
      ? [`${model.submeshes.length} submeshes`, `${model.vertexCount} vertices`, `${triangleCount} triangles`]
      : ["No visual open"];
  }, [isLoading, model, triangleCount]);

  useEditorStatus(status);

  return (
    <EditorLayout
      toolbar={
        <VisualPreviewToolbar
          isOpenEnabled={Boolean(onOpen)}
          hasDetailLevels={hasDetailLevels}
          hasSkeleton={hasSkeleton}
          subtitle={subtitle}
          options={options}
          detail={detail}
          onChangeOptions={setOptions}
          onChangeDetail={setDetail}
          onResetCamera={onResetCamera}
          onOpen={onOpen}
          onBrowse={onBrowse}
        />
      }
      footer={footer}
    >
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        sx={{ position: "relative", display: "flex", flex: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}
      >
        {renderViewport ? (
          renderViewport({ cameraResetToken, detail, hiddenBones, highlightedJoint, model, options, textures })
        ) : (
          <VisualPreviewMotionViewport
            model={model}
            options={options}
            cameraResetToken={cameraResetToken}
            detail={detail}
            highlightedJoint={highlightedJoint}
            hiddenBones={hiddenBones}
            textures={textures}
          />
        )}

        {!model && !isLoading ? <VisualPreviewEmpty error={error} onRetry={onRetry} /> : null}

        {isLoading ? (
          <Box
            sx={{
              position: "absolute",
              inset: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              pointerEvents: "none",
            }}
          >
            <DelayedProgress />
          </Box>
        ) : null}
      </Box>
    </EditorLayout>
  );
}
