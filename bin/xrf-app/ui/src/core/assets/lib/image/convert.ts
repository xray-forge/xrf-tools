/**
 * Decode an object url into an image element.
 *
 * Takes a url rather than a blob because the url outlives this call - the element keeps reading it -
 * so whoever owns its lifetime has to be the one that created it. Creating one here and revoking it on
 * load broke the very element being returned, which is why that revoke sat commented out instead.
 *
 * @param url - Object URL to decode without changing its lifetime.
 * @returns A promise resolving to the decoded image element.
 */
export function urlToImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject): void => {
    const image: HTMLImageElement = new Image();

    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error(`Failed to decode image from ${url}`));

    image.src = url;
  });
}
