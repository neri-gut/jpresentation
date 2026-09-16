import { onUnmounted, ref, watch, type Ref } from "vue";

/**
 * Scale an image to fill its parent without cropping or stretching.
 * Matching aspect fills the box; square on 16:9 letterboxes.
 */
export function useContainFit(imgRef: Ref<HTMLImageElement | null>): {
  fitStyle: Ref<Record<string, string>>;
  layout: () => void;
} {
  const fitStyle = ref<Record<string, string>>({});
  let observer: ResizeObserver | null = null;

  function layout(): void {
    const img = imgRef.value;
    const box = img?.parentElement;
    if (!img || !box || !img.naturalWidth || !img.naturalHeight) {
      return;
    }
    const { clientWidth: cw, clientHeight: ch } = box;
    if (!cw || !ch) {
      return;
    }
    const scale = Math.min(cw / img.naturalWidth, ch / img.naturalHeight);
    if (!Number.isFinite(scale) || scale <= 0) {
      return;
    }
    fitStyle.value = {
      width: `${Math.round(img.naturalWidth * scale)}px`,
      height: `${Math.round(img.naturalHeight * scale)}px`,
    };
  }

  function observe(el: HTMLImageElement | null): void {
    observer?.disconnect();
    observer = null;
    const box = el?.parentElement;
    if (!box || typeof ResizeObserver === "undefined") {
      return;
    }
    observer = new ResizeObserver(() => layout());
    observer.observe(box);
  }

  watch(
    imgRef,
    (el) => {
      observe(el);
      layout();
    },
    { flush: "post" },
  );

  if (typeof window !== "undefined") {
    window.addEventListener("resize", layout);
  }

  onUnmounted(() => {
    observer?.disconnect();
    if (typeof window !== "undefined") {
      window.removeEventListener("resize", layout);
    }
  });

  return { fitStyle, layout };
}
