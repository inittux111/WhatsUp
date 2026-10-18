// WebKitGTK 2.54 runs CSS opacity and transform animations on its
// compositor, and in WhatsApp's media viewer those start and end a frame out
// of step with the page: opening a picture flashes the viewer's black loading
// screen, and zooming flashes the picture at its old size. app.css turns off
// the viewer's own fade-in and zoom transition; this script plays the same
// animations on the page's thread instead, where every frame matches.
(() => {
  const VIEWER = '[data-testid="media-viewer-modal"]';

  // WebKit keeps an animation on the page's thread when it also animates
  // `width` and has a transform that depends on it: a constant width and
  // translateX(0%) change nothing visible but meet that rule.
  const animateOnPage = (element, keyframes, options) => {
    const width = getComputedStyle(element).width;
    const frames = keyframes.map((frame) => ({
      ...frame,
      width,
      transform: `translateX(0%) ${frame.transform && frame.transform !== "none" ? frame.transform : ""}`,
    }));
    return element.animate(frames, options);
  };

  const fadeIn = (viewer) => {
    animateOnPage(viewer, [{ opacity: 0 }, { opacity: 1 }], { duration: 350, easing: "linear" });
  };

  // Zooming sets a new transform on an element whose inline style carries
  // WhatsApp's transition; play that transition from the old transform.
  const shown = new WeakMap();
  const zooms = new WeakMap();
  const zoom = (element) => {
    // The easing ends at the next property (",") or declaration (";"), but
    // may hold commas itself: cubic-bezier(0.1, 0.82, 0.25, 1).
    const transition = /transition:\s*transform\s+(\d+)ms\s+([^;,(]+(?:\([^)]*\))?)/.exec(
      element.getAttribute("style") || "",
    );
    if (!transition) return;
    const target = element.style.transform || "none";
    const before = shown.get(element);
    shown.set(element, target);
    // WhatsApp's own script steps the opening and closing movement frame by
    // frame; only a single new transform is a zoom.
    if (element.classList.contains("velocity-animating")) return;
    if (before === undefined || before === target) return;
    const running = zooms.get(element);
    const from = running?.playState === "running" ? getComputedStyle(element).transform : before;
    running?.cancel();
    const animation = animateOnPage(element, [{ transform: from }, { transform: target }], {
      duration: Number(transition[1]),
      easing: transition[2].trim(),
    });
    zooms.set(element, animation);
  };

  new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "childList") {
        for (const node of mutation.addedNodes) {
          if (node.nodeType !== Node.ELEMENT_NODE) continue;
          const viewer = node.matches(VIEWER) ? node : node.querySelector(VIEWER);
          if (viewer) fadeIn(viewer);
        }
      } else if (mutation.target.closest(VIEWER)) {
        zoom(mutation.target);
      }
    }
  }).observe(document, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["style"],
  });
})();
