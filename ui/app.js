(() => {
  const tauri = window.__TAURI__;
  const appWindow = tauri?.window?.appWindow || null;
  const invoke = tauri?.invoke || (async () => {});

  // DOM Elements
  const appContainer = document.getElementById("app");
  const toolbar = document.getElementById("toolbar");
  const tabBar = document.getElementById("tab-bar");
  const tabListEl = document.getElementById("tab-list");
  const addressForm = document.getElementById("address-form");
  const addressInput = document.getElementById("address-input");
  const heroSearchForm = document.getElementById("hero-search-form");
  const heroSearchInput = document.getElementById("hero-search-input");
  const startPage = document.getElementById("start-page");
  const iframesBox = document.getElementById("iframes-box");

  const btnClose = document.getElementById("btn-close");
  const btnMinimize = document.getElementById("btn-minimize");
  const btnMaximize = document.getElementById("btn-maximize");
  const btnBack = document.getElementById("btn-back");
  const btnForward = document.getElementById("btn-forward");
  const btnHome = document.getElementById("btn-home");
  const btnReload = document.getElementById("btn-reload");
  const btnNewTab = document.getElementById("btn-new-tab");
  const btnStealth = document.getElementById("btn-stealth");
  const stealthText = document.getElementById("stealth-text");

  let tabs = [];
  let activeTabId = null;
  let tabCounter = 1;
  let stealthEnabled = true; // Default aktif saat buka
  let isMaximized = false;
  let isClosing = false;

  function normalizeUrl(input) {
    const raw = (input || "").trim();
    if (!raw || raw === "spectre://start" || raw === "about:blank") return "spectre://start";
    if (/^https?:\/\//i.test(raw)) return raw;
    if (/^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+(\/.*)?$/.test(raw) && !raw.includes(" ")) {
      return "https://" + raw;
    }
    return "https://www.google.com/search?q=" + encodeURIComponent(raw);
  }

  function formatTitle(url) {
    if (!url || url === "spectre://start") return "Halaman Awal";
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch {
      return url;
    }
  }

  function createTab(initialUrl = "spectre://start") {
    const id = "tab-" + tabCounter++;
    const url = normalizeUrl(initialUrl);

    const iframe = document.createElement("iframe");
    iframe.className = "browser-iframe";
    iframe.setAttribute("allow", "autoplay; clipboard-read; clipboard-write; fullscreen; encrypted-media");
    iframesBox.appendChild(iframe);

    const tab = {
      id,
      url,
      title: formatTitle(url),
      history: [url],
      historyIndex: 0,
      iframe,
    };

    tabs.push(tab);
    switchTab(id);
    navigateTab(tab, url, false);
    return tab;
  }

  function switchTab(id) {
    activeTabId = id;
    const activeTab = tabs.find((t) => t.id === id);
    if (!activeTab) return;

    tabs.forEach((t) => {
      t.iframe.classList.toggle("active", t.id === id);
    });

    if (activeTab.url === "spectre://start") {
      startPage.classList.remove("hidden");
      iframesBox.style.display = "none";
      addressInput.value = "";
    } else {
      startPage.classList.add("hidden");
      iframesBox.style.display = "flex";
      addressInput.value = activeTab.url;
    }

    renderTabs();
  }

  function navigateTab(tab, targetUrl, pushHistory = true) {
    const url = normalizeUrl(targetUrl);
    tab.url = url;
    tab.title = formatTitle(url);

    if (pushHistory) {
      tab.history = tab.history.slice(0, tab.historyIndex + 1);
      tab.history.push(url);
      tab.historyIndex = tab.history.length - 1;
    }

    if (url === "spectre://start") {
      tab.iframe.src = "about:blank";
      if (tab.id === activeTabId) {
        startPage.classList.remove("hidden");
        iframesBox.style.display = "none";
        addressInput.value = "";
      }
    } else {
      tab.iframe.src = url;
      if (tab.id === activeTabId) {
        startPage.classList.add("hidden");
        iframesBox.style.display = "flex";
        addressInput.value = url;
      }
    }

    renderTabs();
  }

  function closeTab(id, e) {
    if (e) e.stopPropagation();
    if (tabs.length <= 1) {
      navigateTab(tabs[0], "spectre://start", true);
      return;
    }
    const idx = tabs.findIndex((t) => t.id === id);
    if (idx === -1) return;
    const [removed] = tabs.splice(idx, 1);
    removed.iframe.remove();

    if (activeTabId === id) {
      const nextTab = tabs[Math.max(0, idx - 1)];
      switchTab(nextTab.id);
    } else {
      renderTabs();
    }
  }

  function renderTabs() {
    tabListEl.innerHTML = "";
    tabs.forEach((tab) => {
      const el = document.createElement("div");
      el.className = "tab-item" + (tab.id === activeTabId ? " active" : "");
      el.innerHTML = `
        <span class="tab-title"></span>
        <button class="tab-close" title="Tutup">×</button>
      `;
      el.querySelector(".tab-title").textContent = tab.title || "Tab Baru";
      el.addEventListener("click", () => switchTab(tab.id));
      el.querySelector(".tab-close").addEventListener("click", (ev) => closeTab(tab.id, ev));
      tabListEl.appendChild(el);
    });
  }

  // ================= DRAG WINDOW MENGGUNAKAN HEADER =================
  function setupDrag(element) {
    element.addEventListener("mousedown", async (e) => {
      if (e.button !== 0) return;
      if (e.target.closest("button, input, form, .tab-item")) return;
      if (appWindow) {
        try {
          await appWindow.startDragging();
        } catch {}
      }
    });
  }
  setupDrag(toolbar);
  setupDrag(tabBar);

  // ================= KONTROL BULAT TRAFFIC LIGHTS DENGAN ANIMASI =================
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
    }, 240);
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
    }, 180);
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

  // ================= TOGGLE ANTI-SS (TEMBUS PANDANG DI RECORD/SS, TETAP MUNCUL DI LAYAR) =================
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
    const active = tabs.find((t) => t.id === activeTabId);
    if (active) {
      navigateTab(active, addressInput.value, true);
      addressInput.blur();
    }
  });

  addressInput.addEventListener("focus", () => addressInput.select());

  heroSearchForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const active = tabs.find((t) => t.id === activeTabId);
    if (active && heroSearchInput.value.trim()) {
      navigateTab(active, heroSearchInput.value, true);
      heroSearchInput.value = "";
    }
  });

  document.querySelectorAll(".quick-site-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const url = btn.getAttribute("data-url");
      const active = tabs.find((t) => t.id === activeTabId);
      if (active && url) navigateTab(active, url, true);
    });
  });

  btnBack.addEventListener("click", () => {
    const active = tabs.find((t) => t.id === activeTabId);
    if (active && active.historyIndex > 0) {
      active.historyIndex--;
      navigateTab(active, active.history[active.historyIndex], false);
    }
  });

  btnForward.addEventListener("click", () => {
    const active = tabs.find((t) => t.id === activeTabId);
    if (active && active.historyIndex < active.history.length - 1) {
      active.historyIndex++;
      navigateTab(active, active.history[active.historyIndex], false);
    }
  });

  btnHome.addEventListener("click", () => {
    const active = tabs.find((t) => t.id === activeTabId);
    if (active) navigateTab(active, "spectre://start", true);
  });

  btnReload.addEventListener("click", () => {
    const active = tabs.find((t) => t.id === activeTabId);
    if (active && active.url !== "spectre://start") {
      active.iframe.src = active.url;
    }
  });

  btnNewTab.addEventListener("click", () => {
    createTab("spectre://start");
  });

  window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key.toLowerCase() === "t") {
      e.preventDefault();
      createTab("spectre://start");
    } else if (e.ctrlKey && e.key.toLowerCase() === "w") {
      e.preventDefault();
      if (activeTabId) closeTab(activeTabId);
    }
  });

  // Start with 1 tab
  createTab("spectre://start");
})();
