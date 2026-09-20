import { lockScroll, mount, unlockScroll } from "@cloudflare/nimbus-docs/client";

declare global {
  interface Window {
    __ochubWasmLightboxKeys?: boolean;
  }
}

type LocaleKey = "en" | "zh" | "ja";

const copy = {
  en: {
    close: "Close",
    prev: "Previous step",
    next: "Next step",
    step: "Step",
    of: "of",
    hint: "Esc closes · ← → switches steps",
    path: "Where",
    action: "What to do",
    done: "Done when",
    note: "Watch for",
  },
  zh: {
    close: "关闭",
    prev: "上一步",
    next: "下一步",
    step: "步骤",
    of: "/",
    hint: "Esc 关闭 · ← → 切换步骤",
    path: "位置",
    action: "怎么操作",
    done: "完成标志",
    note: "注意",
  },
  ja: {
    close: "閉じる",
    prev: "前の手順",
    next: "次の手順",
    step: "手順",
    of: "/",
    hint: "Esc で閉じる · ← → で手順を切替",
    path: "場所",
    action: "操作",
    done: "完了の目安",
    note: "注意",
  },
} as const;

const icon = {
  close:
    '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M6.4 17.65 5.35 16.6 10.95 11 5.35 5.4 6.4 4.35 12 9.95 17.6 4.35 18.65 5.4 13.05 11 18.65 16.6 17.6 17.65 12 12.05z"/></svg>',
  prev: '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="m14.7 18.3-6-6.3 6-6.3 1.4 1.35L11.55 12l4.55 4.95z"/></svg>',
  next: '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="m9.3 18.3-1.4-1.35L12.45 12 7.9 7.05 9.3 5.7l6 6.3z"/></svg>',
};

type LightboxState = {
  scene: HTMLElement;
  placeholder: HTMLElement;
  items: HTMLElement[];
  index: number;
};

let state: LightboxState | null = null;
let lastFocus: HTMLElement | null = null;
let shell: HTMLElement | null = null;

const localeOf = (node: Element | null): LocaleKey => {
  const raw =
    node?.closest("[data-locale]")?.getAttribute("data-locale") ??
    document.documentElement.lang ??
    "en";
  if (raw.startsWith("zh")) return "zh";
  if (raw.startsWith("ja")) return "ja";
  return "en";
};

const galleryItems = (scene: HTMLElement): HTMLElement[] => {
  const gallery = scene.closest("[data-wasm-gallery]");
  if (!gallery) return [scene];
  return [...gallery.querySelectorAll<HTMLElement>("[data-wasm-gallery-item] [data-wasm-scene]")];
};

const itemMeta = (scene: HTMLElement) => {
  const item = scene.closest("[data-wasm-gallery-item]");
  return {
    label: item?.getAttribute("data-step-label") ?? scene.querySelector("iframe")?.title ?? "",
    path: item?.getAttribute("data-step-path") ?? "",
    action: item?.getAttribute("data-step-action") ?? "",
    done: item?.getAttribute("data-step-done") ?? "",
    note: item?.getAttribute("data-step-note") ?? "",
  };
};

const ensureShell = (): HTMLElement => {
  if (shell?.isConnected) return shell;
  shell = document.createElement("div");
  shell.className = "wasm-lightbox";
  shell.dataset.wasmLightbox = "";
  shell.setAttribute("hidden", "");
  shell.innerHTML = `
    <div class="wasm-lightbox-backdrop" data-wasm-lightbox-backdrop></div>
    <div class="wasm-lightbox-ui">
      <button type="button" class="wasm-lightbox-close" data-wasm-lightbox-close></button>
      <button type="button" class="wasm-lightbox-nav is-prev" data-wasm-lightbox-prev></button>
      <button type="button" class="wasm-lightbox-nav is-next" data-wasm-lightbox-next></button>
      <div class="wasm-lightbox-caption" data-wasm-lightbox-caption>
        <header>
          <small data-lightbox-kicker></small>
          <strong data-lightbox-title></strong>
        </header>
        <dl data-lightbox-guide></dl>
        <p class="wasm-lightbox-hint" data-lightbox-hint></p>
      </div>
    </div>
  `;
  const closeBtn = shell.querySelector<HTMLButtonElement>("[data-wasm-lightbox-close]")!;
  const prevBtn = shell.querySelector<HTMLButtonElement>("[data-wasm-lightbox-prev]")!;
  const nextBtn = shell.querySelector<HTMLButtonElement>("[data-wasm-lightbox-next]")!;
  closeBtn.innerHTML = icon.close;
  prevBtn.innerHTML = icon.prev;
  nextBtn.innerHTML = icon.next;
  shell.querySelector("[data-wasm-lightbox-backdrop]")?.addEventListener("click", closeLightbox);
  closeBtn.addEventListener("click", closeLightbox);
  prevBtn.addEventListener("click", () => stepLightbox(-1));
  nextBtn.addEventListener("click", () => stepLightbox(1));
  document.body.append(shell);
  return shell;
};

const warmFrame = (scene: HTMLElement | undefined) => {
  const frame = scene?.querySelector<HTMLIFrameElement>("[data-wasm-scene-frame]");
  if (!frame) return;
  if (frame.loading === "lazy") frame.loading = "eager";
};

const renderCaption = (current: HTMLElement, items: HTMLElement[], index: number) => {
  const box = ensureShell();
  const locale = localeOf(current);
  const t = copy[locale];
  const meta = itemMeta(current);
  const closeBtn = box.querySelector<HTMLButtonElement>("[data-wasm-lightbox-close]")!;
  const prevBtn = box.querySelector<HTMLButtonElement>("[data-wasm-lightbox-prev]")!;
  const nextBtn = box.querySelector<HTMLButtonElement>("[data-wasm-lightbox-next]")!;
  const kicker = box.querySelector("[data-lightbox-kicker]");
  const title = box.querySelector("[data-lightbox-title]");
  const guide = box.querySelector("[data-lightbox-guide]");
  const hint = box.querySelector("[data-lightbox-hint]");
  closeBtn.setAttribute("aria-label", t.close);
  prevBtn.setAttribute("aria-label", t.prev);
  nextBtn.setAttribute("aria-label", t.next);
  const many = items.length > 1;
  prevBtn.hidden = !many;
  nextBtn.hidden = !many;
  box.classList.toggle("is-gallery", many);
  if (kicker instanceof HTMLElement) {
    kicker.textContent = many ? `${t.step} ${index + 1} ${t.of} ${items.length}` : "";
    kicker.hidden = !many;
  }
  if (title) title.textContent = meta.label;
  if (guide) {
    const rows: Array<[string, string, string]> = [];
    if (meta.path) rows.push(["path", t.path, meta.path]);
    if (meta.action) rows.push(["action", t.action, meta.action]);
    if (meta.done) rows.push(["done", t.done, meta.done]);
    if (meta.note) rows.push(["note", t.note, meta.note]);
    guide.innerHTML = rows
      .map(
        ([kind, label, value]) =>
          `<div class="${kind === "note" ? "is-note" : ""}"><dt>${label}</dt><dd>${escapeHtml(value)}</dd></div>`,
      )
      .join("");
    guide.toggleAttribute("hidden", rows.length === 0);
  }
  if (hint instanceof HTMLElement) {
    hint.textContent = many ? t.hint : "";
    hint.hidden = !many;
  }
  box.setAttribute("aria-label", meta.label || t.close);
};

const escapeHtml = (value: string) =>
  value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");

const parkScene = (scene: HTMLElement) => {
  scene.classList.remove("is-lightbox");
  scene.removeAttribute("aria-modal");
  scene.removeAttribute("role");
  const placeholder = scene.previousElementSibling;
  if (placeholder instanceof HTMLElement && placeholder.dataset.wasmLightboxPlaceholder !== undefined) {
    placeholder.remove();
  }
};

const promoteScene = (scene: HTMLElement) => {
  if (scene.classList.contains("is-lightbox")) return;
  const placeholder = document.createElement("div");
  placeholder.className = "wasm-lightbox-placeholder";
  placeholder.dataset.wasmLightboxPlaceholder = "";
  const rect = scene.getBoundingClientRect();
  placeholder.style.aspectRatio = `${Math.max(rect.width, 1)} / ${Math.max(rect.height, 1)}`;
  scene.insertAdjacentElement("beforebegin", placeholder);
  scene.classList.add("is-lightbox");
  scene.setAttribute("role", "dialog");
  scene.setAttribute("aria-modal", "true");
  warmFrame(scene);
};

const openLightbox = (scene: HTMLElement) => {
  const items = galleryItems(scene);
  const index = Math.max(0, items.indexOf(scene));
  const current = items[index] ?? scene;
  if (state) parkScene(state.scene);
  lastFocus =
    document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const box = ensureShell();
  box.removeAttribute("hidden");
  box.dataset.open = "true";
  document.documentElement.classList.add("wasm-lightbox-open");
  lockScroll();
  promoteScene(current);
  state = {
    scene: current,
    placeholder: current.previousElementSibling as HTMLElement,
    items,
    index,
  };
  renderCaption(current, items, index);
  warmFrame(items[index + 1]);
  warmFrame(items[index - 1]);
  box.querySelector<HTMLButtonElement>("[data-wasm-lightbox-close]")?.focus();
};

const closeLightbox = () => {
  if (!state) return;
  const scene = state.scene;
  parkScene(scene);
  state = null;
  shell?.setAttribute("hidden", "");
  if (shell) shell.dataset.open = "false";
  document.documentElement.classList.remove("wasm-lightbox-open");
  unlockScroll();
  scene.scrollIntoView({ block: "nearest", inline: "nearest" });
  lastFocus?.focus();
  lastFocus = null;
};

const stepLightbox = (delta: number) => {
  if (!state || state.items.length < 2) return;
  const nextIndex = (state.index + delta + state.items.length) % state.items.length;
  const next = state.items[nextIndex];
  if (!next || next === state.scene) return;
  parkScene(state.scene);
  promoteScene(next);
  state = {
    scene: next,
    placeholder: next.previousElementSibling as HTMLElement,
    items: state.items,
    index: nextIndex,
  };
  renderCaption(next, state.items, nextIndex);
  warmFrame(state.items[nextIndex + delta]);
};

const onKeydown = (event: KeyboardEvent) => {
  if (!state) return;
  if (event.key === "Escape") {
    event.preventDefault();
    closeLightbox();
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    stepLightbox(-1);
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    stepLightbox(1);
    return;
  }
  if (event.key === "Tab" && shell) {
    const buttons = [...shell.querySelectorAll<HTMLButtonElement>("button")].filter(
      (button) => !button.hidden,
    );
    if (buttons.length === 0) return;
    const first = buttons[0];
    const last = buttons[buttons.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  }
};

if (!window.__ochubWasmLightboxKeys) {
  window.__ochubWasmLightboxKeys = true;
  document.addEventListener("keydown", onKeydown);
  document.addEventListener("astro:before-swap", closeLightbox);
}

const scaleDesktopFrame = (scene: HTMLElement, frame: HTMLIFrameElement) => {
  const width = Number(scene.style.getPropertyValue("--scene-width")) || 1280;
  const height = Number(scene.style.getPropertyValue("--scene-height")) || 800;
  const rect = scene.getBoundingClientRect();
  if (rect.width === 0 || rect.height === 0) return;
  frame.style.transform = `scale(${Math.min(1, rect.width / width, rect.height / height)})`;
};

mount("[data-wasm-scene]", (root) => {
  if (!(root instanceof HTMLElement)) return () => {};
  const frame = root.querySelector<HTMLIFrameElement>("[data-wasm-scene-frame]");
  const zoom = root.querySelector<HTMLButtonElement>("[data-wasm-zoom]");
  const onLoad = () => root.setAttribute("data-loaded", "true");
  frame?.addEventListener("load", onLoad, { once: true });
  if (frame?.contentDocument?.readyState === "complete") onLoad();

  let observer: ResizeObserver | undefined;
  if (root.classList.contains("is-desktop") && frame) {
    const scale = () => scaleDesktopFrame(root, frame);
    scale();
    observer = new ResizeObserver(scale);
    observer.observe(root);
  }

  const onZoom = (event: Event) => {
    event.preventDefault();
    event.stopPropagation();
    openLightbox(root);
  };
  zoom?.addEventListener("click", onZoom);

  return () => {
    if (state?.scene === root) closeLightbox();
    observer?.disconnect();
    zoom?.removeEventListener("click", onZoom);
  };
});
