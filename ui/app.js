(() => {
  const tauri = window.__TAURI__;
  const appWindow = tauri?.window?.appWindow || null;
  const invoke = tauri?.invoke || (async () => {});

  const appContainer = document.getElementById("app");
  const toolbar = document.getElementById("toolbar");
  const addressForm = document.getElementById("address-form");
  const addressInput = document.getElementById("address-input");
  const heroSearchForm = document.getElementById("hero-search-form");
  const heroSearchInput = document.getElementById("hero-search-input");

  const btnClose = document.getElementById("btn-close");
  const btnMinimize = document.getElementById("btn-minimize");
  const btnMaximize = document.getElementById("btn-maximize");
  const btnBack = document.getElementById("btn-back");
  const btnForward = document.getElementById("btn-forward");
  const btnHome = document.getElementById("btn-home");
  const btnReload = document.getElementById("btn-reload");
  const btnStealth = document.getElementById("btn-stealth");
  const stealthText = document.getElementById("stealth-text");

  let stealthEnabled = true;
  let isMaximized = false;
  let isClosing = false;

  function normalizeUrl(input) {
    const raw = (input || "").trim();
    if (!raw || raw === "spectre://start" || raw === "about:blank") return "";
    if (/^https?:\/\//i.test(raw)) return raw;
    if (/^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+(\/.*)?$/.test(raw) && !raw.includes(" ")) {
      return "https://" + raw;
    }
    return "https://www.google.com/search?q=" + encodeURIComponent(raw);
  }

  function navigateTo(targetUrl) {
    const url = normalizeUrl(targetUrl);
    if (!url) return;
    window.location.href = url;
  }

  // ================= DRAG WINDOW MENGGUNAKAN HEADER =================
  if (toolbar) {
    toolbar.addEventListener("mousedown", async (e) => {
      if (e.button !== 0) return;
      if (e.target.closest("button, input, form")) return;
      if (appWindow) {
        try {
          await appWindow.startDragging();
        } catch {}
      }
    });
  }

  // ================= KONTROL BULAT TRAFFIC LIGHTS =================
  btnClose.addEventListener("click", () => {
    if (isClosing) return;
    isClosing = true;
    appContainer.classList.add("is-closing");
    setTimeout(async () => {
      if (appWindow) {
        await appWindow.close();
      } else {
        window.close();
      }
    }, 180);
  });

  btnMinimize.addEventListener("click", () => {
    appContainer.classList.add("is-minimizing");
    setTimeout(async () => {
      if (appWindow) {
        await appWindow.minimize();
      }
      setTimeout(() => {
        appContainer.classList.remove("is-minimizing");
      }, 150);
    }, 150);
  });

  btnMaximize.addEventListener("click", async () => {
    if (appWindow) {
      if (isMaximized) {
        await appWindow.unmaximize();
        isMaximized = false;
      } else {
        await appWindow.maximize();
        isMaximized = true;
      }
    }
  });

  // ================= TOGGLE ANTI-SS =================
  btnStealth.addEventListener("click", async () => {
    stealthEnabled = !stealthEnabled;
    try {
      await invoke("set_stealth_mode", { enabled: stealthEnabled });
    } catch {}

    btnStealth.classList.toggle("off", !stealthEnabled);
    stealthText.textContent = stealthEnabled ? "Anti-SS: ON" : "Anti-SS: OFF";
  });

  // ================= NAVIGASI =================
  addressForm.addEventListener("submit", (e) => {
    e.preventDefault();
    navigateTo(addressInput.value);
  });

  addressInput.addEventListener("focus", () => addressInput.select());

  heroSearchForm.addEventListener("submit", (e) => {
    e.preventDefault();
    if (heroSearchInput.value.trim()) {
      navigateTo(heroSearchInput.value);
    }
  });

  document.querySelectorAll(".quick-site-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const url = btn.getAttribute("data-url");
      if (url) navigateTo(url);
    });
  });

  btnBack.addEventListener("click", () => window.history.back());
  btnForward.addEventListener("click", () => window.history.forward());
  btnHome.addEventListener("click", () => window.location.href = "https://tauri.localhost/");
  btnReload.addEventListener("click", () => window.location.reload());
})();
