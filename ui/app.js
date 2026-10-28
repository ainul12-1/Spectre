(() => {
  const tauri = window.__TAURI__;
  const appWindow = tauri?.window?.appWindow || null;
  const invoke = tauri?.invoke || (async () => {});
  const listen = tauri?.event?.listen || (async () => {});

  // DOM Elements
  const safariShell = document.getElementById("safari-shell");
  const safariHeader = document.getElementById("safari-header");
  const tabBar = document.getElementById("tab-bar");
  const tabListEl = document.getElementById("tab-list");
  const addressForm = document.getElementById("address-form");
  const addressInput = document.getElementById("address-input");
  const heroSearchForm = document.getElementById("hero-search-form");
  const heroSearchInput = document.getElementById("hero-search-input");
  const startPage = document.getElementById("start-page");
  const iframesContainer = document.getElementById("iframes-container");
  const dragShield = document.getElementById("drag-shield");

  const btnClose = document.getElementById("btn-close");
  const btnMinimize = document.getElementById("btn-minimize");
  const btnMaximize = document.getElementById("btn-maximize");
  const btnBack = document.getElementById("btn-back");
  const btnForward = document.getElementById("btn-forward");
  const btnHome = document.getElementById("btn-home");
  const btnReload = document.getElementById("btn-reload");
  const btnNewTab = document.getElementById("btn-new-tab");
  const btnStealth = document.getElementById("btn-stealth");
  const stealthLabel = document.getElementById("stealth-label");
  const btnWobbleTest = document.getElementById("btn-wobble-test");
  const btnSettings = document.getElementById("btn-settings");
  const settingsPopover = document.getElementById("settings-popover");

  const opacitySlider = document.getElementById("opacity-slider");
  const opacityVal = document.getElementById("opacity-val");
  const clothSlider = document.getElementById("cloth-slider");
  const clothVal = document.getElementById("cloth-val");
  const chkAlwaysOnTop = document.getElementById("chk-always-on-top");
  const resizeHandle = document.getElementById("resize-handle");

  const svgDisplacement = document.getElementById("cloth-displacement");
  const svgTurbulence = document.getElementById("cloth-turbulence");

  // ================= STATE BROWSER & MULTI-TAB =================
  let tabs = [];
  let activeTabId = null;
  let tabCounter = 1;
  let stealthEnabled = true;
  let isMaximized = false;
  let isAnimatingWindowAction = false;

  function normalizeUrl(input) {
    const raw = (input || "").trim();
    if (!raw) return "spectre://start";
    if (raw === "spectre://start" || raw === "about:blank") return "spectre://start";
    if (/^https?:\/\//i.test(raw)) return raw;
    // Cek apakah terlihat seperti domain (misal google.com, github.com/user)
    if (/^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+(\/.*)?$/.test(raw) && !raw.includes(" ")) {
      return "https://" + raw;
    }
    return "https://www.google.com/search?q=" + encodeURIComponent(raw);
  }

  function formatDisplayTitle(url) {
    if (!url || url === "spectre://start") return "Halaman Awal";
    try {
      const u = new URL(url);
      return u.hostname.replace(/^www\./, "");
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
    iframe.setAttribute("referrerpolicy", "no-referrer-when-downgrade");
    iframesContainer.appendChild(iframe);

    const tabObj = {
      id,
      url,
      title: formatDisplayTitle(url),
      history: [url],
      historyIndex: 0,
      iframe,
    };

    iframe.addEventListener("load", () => {
      try {
        const currentHref = iframe.contentWindow.location.href;
        if (currentHref && currentHref !== "about:blank") {
          tabObj.url = currentHref;
          if (tabObj.id === activeTabId) {
            addressInput.value = currentHref;
          }
        }
        const docTitle = iframe.contentDocument?.title;
        if (docTitle) {
          tabObj.title = docTitle;
          renderTabs();
        }
      } catch {
        // Cross-origin fallback
      }
    });

    tabs.push(tabObj);
    switchTab(id);
    navigateTab(tabObj, url, false);
    return tabObj;
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
      iframesContainer.classList.add("hidden");
      addressInput.value = "";
    } else {
      startPage.classList.add("hidden");
      iframesContainer.classList.remove("hidden");
      addressInput.value = activeTab.url;
    }

    renderTabs();
  }

  function navigateTab(tab, targetInput, pushHistory = true) {
    const url = normalizeUrl(targetInput);
    tab.url = url;
    tab.title = formatDisplayTitle(url);

    if (pushHistory) {
      tab.history = tab.history.slice(0, tab.historyIndex + 1);
      tab.history.push(url);
      tab.historyIndex = tab.history.length - 1;
    }

    if (url === "spectre://start") {
      tab.iframe.src = "about:blank";
      if (tab.id === activeTabId) {
        startPage.classList.remove("hidden");
        iframesContainer.classList.add("hidden");
        addressInput.value = "";
      }
    } else {
      tab.iframe.src = url;
      if (tab.id === activeTabId) {
        startPage.classList.add("hidden");
        iframesContainer.classList.remove("hidden");
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
      const btn = document.createElement("div");
      btn.className = "safari-tab" + (tab.id === activeTabId ? " active" : "");
      btn.innerHTML = `
        <span class="tab-title"></span>
        <button class="tab-close" title="Tutup Tab">×</button>
      `;
      btn.querySelector(".tab-title").textContent = tab.title || "Tab Baru";
      btn.addEventListener("click", () => switchTab(tab.id));
      btn.querySelector(".tab-close").addEventListener("click", (ev) => closeTab(tab.id, ev));
      tabListEl.appendChild(btn);
    });
  }

  // ================= ENGINE FISIKA KAIN LENTUR (CLOTH SPRING PHYSICS) =================
  let clothStrength = 1.25; // Default 125%
  let clothX = 0;
  let clothY = 0;
  let velX = 0;
  let velY = 0;
  let wavePhase = 0;
  let filterActive = false;

  let lastWinX = window.screenX;
  let lastWinY = window.screenY;

  function applyClothImpulse(dx, dy) {
    if (clothStrength <= 0.01 || isAnimatingWindowAction) return;
    const clampedDx = Math.max(-65, Math.min(65, dx));
    const clampedDy = Math.max(-45, Math.min(45, dy));
    velX += clampedDx * 0.42 * clothStrength;
    velY += clampedDy * 0.32 * clothStrength;
  }

  // Terima event perpindahan window langsung dari Rust
  listen("spectre://window-moved", (event) => {
    const { x, y } = event.payload;
    if (typeof lastWinX === "number" && typeof lastWinY === "number") {
      const dx = x - lastWinX;
      const dy = y - lastWinY;
      if (Math.abs(dx) < 250 && Math.abs(dy) < 250) {
        applyClothImpulse(dx, dy);
      }
    }
    lastWinX = x;
    lastWinY = y;
  });

  function updateClothPhysics() {
    // Deteksi cadangan via window.screenX / screenY
    const curScreenX = window.screenX;
    const curScreenY = window.screenY;
    if (typeof curScreenX === "number" && typeof lastWinX === "number") {
      const sdx = curScreenX - lastWinX;
      const sdy = curScreenY - lastWinY;
      if ((sdx !== 0 || sdy !== 0) && Math.abs(sdx) < 250 && Math.abs(sdy) < 250) {
        applyClothImpulse(sdx, sdy);
        lastWinX = curScreenX;
        lastWinY = curScreenY;
      }
    }

    if (!isAnimatingWindowAction) {
      // Konstanta Pegas Kain (Hooke's Law + Damping)
      const stiffness = 0.14;
      const damping = 0.80;

      const forceX = -stiffness * clothX;
      const forceY = -stiffness * clothY;

      velX = (velX + forceX) * damping;
      velY = (velY + forceY) * damping;

      clothX += velX;
      clothY += velY;

      const speed = Math.hypot(clothX, clothY, velX, velY);

      if (speed > 0.04) {
        wavePhase += 0.18;

        // Saat ditarik ke kanan (clothX > 0), bagian bawah kain tertinggal ke kiri (skewX & rotZ)
        const skewX = Math.max(-16, Math.min(16, clothX * 0.45));
        const skewY = Math.max(-6, Math.min(6, clothY * 0.18));
        const rotZ = Math.max(-8, Math.min(8, -clothX * 0.22));
        const rotY = Math.max(-20, Math.min(20, clothX * 0.52));
        const rotX = Math.max(-14, Math.min(14, -clothY * 0.42));

        const stretchX = 1 + Math.min(0.045, Math.abs(clothX) * 0.0014);
        const squashY = 1 - Math.min(0.03, Math.abs(clothX) * 0.0009);

        safariShell.style.transform = `
          perspective(1050px)
          rotateY(${rotY.toFixed(2)}deg)
          rotateX(${rotX.toFixed(2)}deg)
          rotateZ(${rotZ.toFixed(2)}deg)
          skew(${skewX.toFixed(2)}deg, ${skewY.toFixed(2)}deg)
          scale(${stretchX.toFixed(4)}, ${squashY.toFixed(4)})
        `;

        // Gelombang SVG Displacement agar benar-benar bergelombang seperti kain
        const dispScale = Math.min(18, speed * 0.65 * clothStrength);
        if (dispScale > 0.8) {
          if (!filterActive) {
            safariShell.style.filter = "url(#cloth-wave-filter)";
            filterActive = true;
          }
          svgDisplacement.setAttribute("scale", dispScale.toFixed(1));
          const freqX = (0.011 + Math.sin(wavePhase) * 0.003).toFixed(4);
          const freqY = (0.017 + Math.cos(wavePhase) * 0.003).toFixed(4);
          svgTurbulence.setAttribute("baseFrequency", `${freqX} ${freqY}`);
        } else if (filterActive) {
          svgDisplacement.setAttribute("scale", "0");
          safariShell.style.filter = "none";
          filterActive = false;
        }
      } else {
        clothX = 0;
        clothY = 0;
        velX = 0;
        velY = 0;
        safariShell.style.transform = "none";
        if (filterActive) {
          svgDisplacement.setAttribute("scale", "0");
          safariShell.style.filter = "none";
          filterActive = false;
        }
      }
    }

    requestAnimationFrame(updateClothPhysics);
  }

  requestAnimationFrame(updateClothPhysics);

  // ================= DRAG HEADER SAFARI =================
  function setupHeaderDrag(element) {
    element.addEventListener("mousedown", async (e) => {
      if (e.button !== 0) return;
      if (e.target.closest(".no-drag")) return;

      settingsPopover.classList.add("hidden");

      if (appWindow) {
        dragShield.classList.remove("hidden");
        try {
          await appWindow.startDragging();
        } catch {
          // Fallback
        } finally {
          setTimeout(() => dragShield.classList.add("hidden"), 120);
        }
      } else {
        // Mode preview di browser biasa
        let prevX = e.clientX;
        let prevY = e.clientY;
        const onMove = (me) => {
          applyClothImpulse(me.clientX - prevX, me.clientY - prevY);
          prevX = me.clientX;
          prevY = me.clientY;
        };
        const onUp = () => {
          window.removeEventListener("mousemove", onMove);
          window.removeEventListener("mouseup", onUp);
        };
        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
      }
    });
  }

  setupHeaderDrag(safariHeader);
  setupHeaderDrag(tabBar);

  // Resize Handle Pojok Kanan Bawah
  resizeHandle.addEventListener("mousedown", async (e) => {
    e.stopPropagation();
    if (appWindow && appWindow.startResizeDragging) {
      try {
        await appWindow.startResizeDragging("SouthEast");
      } catch {}
    }
  });

  // ================= KONTROL TRAFFIC LIGHTS (CLOSE / MINIMIZE / MAXIMIZE) =================
  btnClose.addEventListener("click", () => {
    if (isAnimatingWindowAction) return;
    isAnimatingWindowAction = true;
    safariShell.style.filter = "none";
    safariShell.classList.add("is-closing");

    setTimeout(async () => {
      if (appWindow) {
        await appWindow.close();
      } else {
        window.close();
      }
    }, 390);
  });

  btnMinimize.addEventListener("click", () => {
    if (isAnimatingWindowAction) return;
    isAnimatingWindowAction = true;
    safariShell.style.filter = "none";
    safariShell.classList.add("is-minimizing");

    setTimeout(async () => {
      if (appWindow) {
        await appWindow.minimize();
      }
      setTimeout(() => {
        safariShell.classList.remove("is-minimizing");
        isAnimatingWindowAction = false;
      }, 250);
    }, 340);
  });

  btnMaximize.addEventListener("click", async () => {
    // Efek membal kain saat di-maximize/restore
    applyClothImpulse(28, -18);
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

  // ================= KONTROL NAVIGASI BROWSER =================
  addressForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (activeTab) {
      navigateTab(activeTab, addressInput.value, true);
      addressInput.blur();
    }
  });

  addressInput.addEventListener("focus", () => {
    addressInput.select();
  });

  heroSearchForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (activeTab && heroSearchInput.value.trim()) {
      navigateTab(activeTab, heroSearchInput.value, true);
      heroSearchInput.value = "";
    }
  });

  document.querySelectorAll(".quick-card").forEach((card) => {
    card.addEventListener("click", () => {
      const targetUrl = card.getAttribute("data-url");
      const activeTab = tabs.find((t) => t.id === activeTabId);
      if (activeTab && targetUrl) {
        navigateTab(activeTab, targetUrl, true);
      }
    });
  });

  btnBack.addEventListener("click", () => {
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (!activeTab) return;
    if (activeTab.historyIndex > 0) {
      activeTab.historyIndex--;
      navigateTab(activeTab, activeTab.history[activeTab.historyIndex], false);
    }
  });

  btnForward.addEventListener("click", () => {
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (!activeTab) return;
    if (activeTab.historyIndex < activeTab.history.length - 1) {
      activeTab.historyIndex++;
      navigateTab(activeTab, activeTab.history[activeTab.historyIndex], false);
    }
  });

  btnHome.addEventListener("click", () => {
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (activeTab) {
      navigateTab(activeTab, "spectre://start", true);
    }
  });

  btnReload.addEventListener("click", () => {
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (!activeTab) return;
    if (activeTab.url !== "spectre://start") {
      activeTab.iframe.src = activeTab.url;
    }
  });

  btnNewTab.addEventListener("click", () => {
    createTab("spectre://start");
  });

  // ================= FITUR STEALTH (ANTI-SS) & POPOVER =================
  btnStealth.addEventListener("click", async () => {
    stealthEnabled = !stealthEnabled;
    try {
      await invoke("set_stealth_mode", { enabled: stealthEnabled });
    } catch {}
    btnStealth.classList.toggle("off", !stealthEnabled);
    stealthLabel.textContent = stealthEnabled ? "Anti-SS: ON" : "Anti-SS: OFF";
  });

  btnWobbleTest.addEventListener("click", () => {
    // Berikan goyangan kain kanan-kiri otomatis untuk demo visual
    applyClothImpulse(52, -14);
    setTimeout(() => applyClothImpulse(-64, 16), 110);
    setTimeout(() => applyClothImpulse(35, -8), 230);
  });

  btnSettings.addEventListener("click", (e) => {
    e.stopPropagation();
    settingsPopover.classList.toggle("hidden");
  });

  opacitySlider.addEventListener("input", () => {
    const val = Number(opacitySlider.value);
    opacityVal.textContent = `${val}%`;
    safariShell.style.opacity = (val / 100).toFixed(2);
  });

  clothSlider.addEventListener("input", () => {
    const val = Number(clothSlider.value);
    clothVal.textContent = `${val}%`;
    clothStrength = val / 100;
    applyClothImpulse(25, 0);
  });

  chkAlwaysOnTop.addEventListener("change", async () => {
    try {
      await invoke("set_always_on_top", { enabled: chkAlwaysOnTop.checked });
    } catch {}
  });

  // Shortcut Keyboard (Ctrl+T = Tab Baru, Ctrl+W = Tutup Tab)
  window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key.toLowerCase() === "t") {
      e.preventDefault();
      createTab("spectre://start");
    } else if (e.ctrlKey && e.key.toLowerCase() === "w") {
      e.preventDefault();
      if (activeTabId) closeTab(activeTabId);
    }
  });

  // Buat tab pertama saat aplikasi menyala
  createTab("spectre://start");
})();
