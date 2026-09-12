import { useInjection } from "@wirestate/react";
import { useEffect, useMemo, useState } from "react";

import { AssetService } from "@/core/assets/services";
import { Nullable } from "@/lib/types/general";

/**
 * The url a preview shows, kept under one key so the last picture is released when the next one arrives.
 *
 * Blobbed from the view rather than its buffer, so a byte offset cannot silently widen the picture.
 *
 * The key is released when the picture goes away as well as when it is replaced: leaving an editor unmounts the preview
 * without deactivating the application it belongs to, and a url nobody revokes there keeps its bytes - and its entry in
 * the webview's source list - for as long as the window lives.
 *
 * @param key - Slot the url is held in. One key per picture on screen: a second pane comparing two of them needs two,
 *   or each would revoke the other's url as it arrived.
 * @param bytes - The encoded bytes, as a buffer or a view of one, or null when there is nothing to show.
 * @param type - Mime type of those bytes.
 * @returns The url to point an `img` at, or null while there is nothing to show.
 */
export function useAssetUrl(key: string, bytes: Nullable<BlobPart>, type: string = "image/png"): Nullable<string> {
  const assetService: AssetService = useInjection(AssetService);

  const [url, setUrl] = useState<Nullable<string>>(null);

  const blob: Nullable<Blob> = useMemo(() => (bytes ? new Blob([bytes], { type }) : null), [bytes, type]);

  useEffect(() => {
    if (blob) {
      setUrl(assetService.swap(key, blob));
    } else {
      assetService.releaseKey(key);
      setUrl(null);
    }
  }, [assetService, blob, key]);

  // Released on the way out only, not beside the swap above: swapping replaces in place and keeps the old url alive
  // until its replacement exists.
  useEffect(() => () => assetService.releaseKey(key), [assetService, key]);

  return url;
}
