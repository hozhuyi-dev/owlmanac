// pdf.js, its worker and the text layer's CSS. Imported only on demand (the About page's resume), so none of
// it is in any other page's bundle.
import { AbortException, GlobalWorkerOptions, RenderingCancelledException, TextLayer, getDocument } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import type { PDFPageProxy } from "pdfjs-dist";
import "./pdf-text-layer.css";

GlobalWorkerOptions.workerSrc = workerUrl;

/** A PDF's pages, read from its URL. */
export async function openPdf(url: string): Promise<PDFPageProxy[]> {
  const pdf = await getDocument({ url }).promise;
  return Promise.all(Array.from({ length: pdf.numPages }, (_, i) => pdf.getPage(i + 1)));
}

/** A link on a page: where it goes, and where it sits, in % of the page so it follows the page's width. */
export interface PageLink {
  url: string;
  left: number;
  top: number;
  width: number;
  height: number;
}

/**
 * A page's links to other places (web pages, email), read from its annotations. The drawing is only a
 * picture, so these are laid over it as real links. Links within the PDF itself are left out.
 */
export async function pageLinks(page: PDFPageProxy): Promise<PageLink[]> {
  const viewport = page.getViewport({ scale: 1 });
  const annotations: Array<{ subtype?: string; url?: string; rect?: number[] }> = await page.getAnnotations({ intent: "display" });
  return annotations.flatMap((a) => {
    if (a.subtype !== "Link" || !a.url || !a.rect) return [];
    // PDF space runs up from the page's foot; the viewport's runs down from its top.
    const [x1, y1] = viewport.convertToViewportPoint(a.rect[0], a.rect[1]);
    const [x2, y2] = viewport.convertToViewportPoint(a.rect[2], a.rect[3]);
    const left = Math.min(x1, x2);
    const top = Math.min(y1, y2);
    return [
      {
        url: a.url,
        left: (left / viewport.width) * 100,
        top: (top / viewport.height) * 100,
        width: (Math.abs(x2 - x1) / viewport.width) * 100,
        height: (Math.abs(y2 - y1) / viewport.height) * 100,
      },
    ];
  });
}

/** A page's size in PDF units, as its text layer is laid out. */
export function pageSize(page: PDFPageProxy): { width: number; height: number } {
  const { width, height } = page.getViewport({ scale: 1 });
  return { width, height };
}

/**
 * Draws a page `pixels` wide into the canvas, and its text (to select and copy, and for screen readers) into
 * the layer over it. The layer sizes itself by --total-scale-factor, so it follows the page's width without
 * being drawn again. `cancel` stops both; `done` is then false.
 */
export function drawPage(page: PDFPageProxy, canvas: HTMLCanvasElement, textLayer: HTMLDivElement, pixels: number) {
  const viewport = page.getViewport({ scale: pixels / pageSize(page).width });
  canvas.width = Math.floor(viewport.width);
  canvas.height = Math.floor(viewport.height);
  const render = page.render({ canvas, viewport });

  textLayer.replaceChildren();
  const text = new TextLayer({ textContentSource: page.streamTextContent(), container: textLayer, viewport });

  return {
    done: Promise.all([render.promise, text.render()]).then(
      () => true,
      (error: unknown) => {
        if (error instanceof RenderingCancelledException || error instanceof AbortException) return false;
        throw error;
      },
    ),
    cancel: () => {
      render.cancel();
      text.cancel();
    },
  };
}
