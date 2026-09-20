import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { ReactElement, ReactNode, useMemo, useState } from "react";
import { Texture } from "three";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { isAlphaRenderSurface } from "@/core/render/lib/surface/render-surface";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import {
  IVisualPreviewViewportProps,
  VisualPreviewEmpty,
  VisualPreviewMotionViewport,
  VisualPreviewToolbar,
} from "@/core/visuals/components/preview";
import {
  DEFAULT_VISUAL_LIGHTING,
  DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS,
  IVisualPreviewViewOptions,
} from "@/core/visuals/lib/scene";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { countVisualTriangles, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IVisualPreviewLayoutProps extends BaseComponentProps {
  /** The model on screen, or null while nothing is open. */
  model?: Nullable<IVisualModelViews>;
  /** Shown in the toolbar beside the view toggles, usually where the model came from. */
  subtitle?: ReactNode;
  /** What the open visual is called. Its presence is what draws the file header over the viewport. */
  name?: Nullable<string>;
  /** Published as a left panel when given. Opening a single visual has nothing to browse. */
  tree?: ReactNode;
  /** Data panels the owning application contributes to the right stripe. */
  panels?: Array<IEditorPanel>;
  /** Loaded textures by submesh index, passed straight through to the viewport. */
  textures?: ReadonlyMap<number, Texture>;
  /** Loaded bump pairs by submesh index, passed straight through to the viewport; any at all offers the bump toggle. */
  bumps?: ReadonlyMap<number, IVisualBumpTextures>;
  /** Joint to mark in the viewport, named elsewhere - the bones panel - and resolved to a position by its owner. */
  highlightedJoint?: Nullable<[number, number, number]>;
  /** Bones the viewport collapses, by index, as the engine collapses an addon that is not attached. */
  hiddenBones?: ReadonlySet<number>;
  /**
   * Draws the viewport, for a surface that poses the model from something other than a single picked motion.
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
  onBack?: () => void;
  /** Promotes a single-model session to a browsed one. Absent while already browsing. */
  onBrowse?: () => void;
  /** Ends the selection without ending the session, which is what puts the close action in the file header. */
  onDeselect?: Nullable<() => void>;
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
  name = null,
  tree,
  panels,
  textures,
  bumps,
  highlightedJoint = null,
  hiddenBones,
  renderViewport,
  footer,
  isLoading = false,
  error,
  onRetry,
  onBack,
  onBrowse,
  onDeselect = null,
}: IVisualPreviewLayoutProps): ReactElement {
  const [options, setOptions] = useState<IVisualPreviewViewOptions>(DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS);
  const [lighting, setLighting] = useState<IRenderLighting>(DEFAULT_VISUAL_LIGHTING);
  const [detail, setDetail] = useState(0);

  /**
   * Detail is a fraction of each submesh's collapse chain, so it needs no clamping and survives a model change
   * meaningfully: stepping through a tree at half detail stays at half detail whether the next model carries four
   * collapse steps or nine hundred.
   */
  const triangleCount: number = model ? countVisualTriangles(model, detail) : 0;
  const hasDetailLevels: boolean = (model?.levelCount ?? 1) > 1;
  const hasSkeleton: boolean = Boolean(model?.skeleton);
  // A dummy pair counts: it is uploaded and shaded, and comparing it flat is how a modder sees that it adds nothing.
  const hasBump: boolean = Boolean(model && bumps && bumps.size > 0);
  const hasAlpha: boolean = Boolean(model?.submeshes.some((submesh) => isAlphaRenderSurface(submesh.surface)));

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
          hasDetailLevels={hasDetailLevels}
          hasSkeleton={hasSkeleton}
          hasBump={hasBump}
          hasAlpha={hasAlpha}
          subtitle={subtitle}
          options={options}
          lighting={lighting}
          detail={detail}
          onChangeOptions={setOptions}
          onChangeLighting={setLighting}
          onChangeDetail={setDetail}
          onBack={onBack}
          onBrowse={onBrowse}
        />
      }
      footer={footer}
    >
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {name ? (
          <EditorFileHeader
            data-testid={"visual-file-header"}
            name={name}
            closeLabel={"Close visual"}
            closeDescription={"Clear the selection and close this visual"}
            onClose={onDeselect ?? undefined}
          />
        ) : null}

        <div
          data-testid={dataTestId}
          id={id}
          className={cn("relative flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}
        >
          {renderViewport ? (
            renderViewport({ bumps, detail, hiddenBones, highlightedJoint, lighting, model, options, textures })
          ) : (
            <VisualPreviewMotionViewport
              model={model}
              options={options}
              lighting={lighting}
              detail={detail}
              highlightedJoint={highlightedJoint}
              hiddenBones={hiddenBones}
              textures={textures}
              bumps={bumps}
            />
          )}

          {!model && !isLoading ? <VisualPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isLoading ? (
            <div className={"pointer-events-none absolute inset-0 flex items-center justify-center"}>
              <DelayedProgress isOnViewport={true} label={"Loading visual…"} />
            </div>
          ) : null}
        </div>
      </div>
    </EditorLayout>
  );
}
