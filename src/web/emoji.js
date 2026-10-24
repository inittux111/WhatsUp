// WebKitGTK 2.54 breaks a WebP image for good when it paints the image before
// all of its data has arrived: it stays blank wherever it is used, until the
// page is reloaded. WhatsApp's emoji are cut from sprite sheets set as CSS
// backgrounds, and opening the emoji picker loads and paints dozens of sheets
// at once, so random groups of emoji were missing, in chats as well.
// Each sheet is kept off the page until an unpainted image has loaded it
// completely; from then on the page paints that complete copy from WebKit's
// memory cache.
(() => {
  const SPRITE = /^url\("(https:\/\/web\.whatsapp\.com\/emoji\/[^"]*\/sprite\/[^"]*\.webp)"\)$/;
  // The images keep the loaded sheets in WebKit's memory cache.
  const loaded = new Map();
  const loading = new Map(); // sheet URL -> the selectors that use it
  let hide = null;

  const update = () => {
    if (!hide) {
      hide = document.createElement("style");
      document.documentElement.append(hide);
    }
    hide.textContent = [...loading.values()]
      .flat()
      .map((selector) => `${selector} { background-image: none !important; }`)
      .join("\n");
  };

  const hold = (rule) => {
    const url = rule.style?.backgroundImage.match(SPRITE)?.[1];
    if (!url || loaded.has(url)) return;
    const selectors = loading.get(url);
    if (selectors) {
      selectors.push(rule.selectorText);
    } else {
      loading.set(url, [rule.selectorText]);
      const image = new Image();
      image.onload = image.onerror = () => {
        loading.delete(url);
        loaded.set(url, image);
        update();
      };
      image.src = url;
    }
    update();
  };

  // WhatsApp adds the sheets' rules one by one into its own stylesheet.
  const { insertRule } = CSSStyleSheet.prototype;
  CSSStyleSheet.prototype.insertRule = function (...args) {
    const index = insertRule.apply(this, args);
    try {
      hold(this.cssRules[index]);
    } catch {
      // Never let the fix break WhatsApp's own styling.
    }
    return index;
  };
})();
