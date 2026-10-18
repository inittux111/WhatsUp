// WebKitGTK 2.54 garbles media it reads from blob: URLs (larger files lose
// data on the way to GStreamer), so WhatsApp's videos stay black and silent.
// GStreamer reads data: URLs itself, so media gets a data: URL with the same
// contents instead. Reading `src` still gives the page its own blob: URL.
(() => {
  const proto = HTMLMediaElement.prototype;
  const src = Object.getOwnPropertyDescriptor(proto, "src");
  const { play, pause, load } = proto;
  // Per element while its data: URL is being made: the blob: URL the page
  // set, the conversion, and whether the page wants it playing by then.
  const pending = new WeakMap();
  const given = new WeakMap();

  const toDataUrl = async (url) => {
    const blob = await (await fetch(url)).blob();
    const typed = blob.type ? blob : new Blob([blob], { type: "video/mp4" });
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result);
      reader.onerror = () => reject(reader.error);
      reader.readAsDataURL(typed);
    });
  };

  Object.defineProperty(proto, "src", {
    configurable: true,
    enumerable: src.enumerable,
    get() {
      return given.get(this) ?? src.get.call(this);
    },
    set(value) {
      const url = String(value);
      pending.delete(this);
      if (!url.startsWith("blob:")) {
        given.delete(this);
        src.set.call(this, value);
        return;
      }
      given.set(this, url);
      const state = { wantsPlay: false };
      state.done = toDataUrl(url)
        .catch(() => url)
        .then((data) => {
          if (pending.get(this) !== state) return;
          pending.delete(this);
          src.set.call(this, data);
          if (state.wantsPlay) return play.call(this);
        });
      pending.set(this, state);
    },
  });

  // WhatsApp sets the attribute rather than the property.
  const { setAttribute } = Element.prototype;
  Element.prototype.setAttribute = function (name, value) {
    if (this instanceof HTMLMediaElement && String(name).toLowerCase() === "src") {
      this.src = value;
      return;
    }
    return setAttribute.call(this, name, value);
  };

  proto.play = function () {
    const state = pending.get(this);
    if (!state) return play.call(this);
    state.wantsPlay = true;
    return state.done;
  };
  proto.pause = function () {
    const state = pending.get(this);
    if (state) state.wantsPlay = false;
    return pause.call(this);
  };
  // Setting the data: URL loads it; loading the old source first would not.
  proto.load = function () {
    if (!pending.has(this)) return load.call(this);
  };
})();
