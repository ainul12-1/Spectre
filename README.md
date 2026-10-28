# 👻 Spectre — Stealth Safari Browser (Anti-SS / Anti-Record + Cloth Physics)

**Spectre** adalah browser siluman (*stealth browser*) super ringan berbasis **Rust + Tauri** untuk Windows 10 & Windows 11.

---

## ✨ Fitur Utama

1. **🛡️ Anti-Screenshot & Anti-Screen Record (`WDA_EXCLUDEFROMCAPTURE`)**
   - Menggunakan Win32 API `SetWindowDisplayAffinity(hwnd, 0x00000011)`.
   - **Tetap terlihat 100% normal di mata kamu**, tapi **100% tembus pandang (transparan)** di hasil Screenshot (`PrintScreen`, `Win+Shift+S`, Lightshot) maupun rekaman layar / share screen (`OBS`, `Zoom`, `Google Meet`, `Discord`, `Teams`). Aplikasi atau tab di belakang Spectre yang akan terlihat di hasil tangkapan layar.
2. **🌊 Efek Kain Lentur (Cloth / Wobbly Window Physics)**
   - Saat bagian atas window (Header Safari) di-drag ke kanan/kiri/atas/bawah, window akan melengkung, miring, dan bergelombang lentur seperti kain mengikuti kecepatan gerakan mouse lalu membal halus saat berhenti.
   - Dilengkapi tombol **Tes Goyangan Kain** dan slider **Kelenturan Kain (`0% - 200%`)**.
3. **🍎 UI Ala Safari macOS + Animasi Close**
   - Frameless window dengan 3 tombol bulat warna-warni (*Traffic Lights*: 🔴 Merah, 🟡 Kuning, 🟢 Hijau).
   - Saat tombol 🔴 **Close** atau 🟡 **Minimize** diklik, window menyusut & melipat halus (efek *Cloth Fold / Genie*) sebelum tertutup.
   - Mendukung **Multi-Tab**, Smart Address Bar (otomatis mendeteksi URL atau pencarian Google), serta bypass `X-Frame-Options` sehingga situs seperti Google, YouTube, Wikipedia, Bing, dll dapat dibuka langsung di dalam tab.
4. **☁️ 100% Build di GitHub Actions (Tanpa Install Compiler di PC)**
   - Sudah dilengkapi workflow `.github/workflows/build.yml` yang otomatis meng-compile **`Spectre-Portable.exe`** (ukuran ~3 MB) dan Installer di server GitHub.

---

## 🚀 Cara Build `.exe` di GitHub (Tanpa Install Apa-Apa di PC)

1. Buat repository baru di GitHub (misalnya bernama `spectre-browser`).
2. Upload atau push isi folder `Spectre` ini ke repository GitHub kamu:
   - **Cara via Git (jika ada Git):**
     ```powershell
     cd "c:\Users\safaat\Documents\app windows\Spectre"
     git init
     git branch -M main
     git add .
     git commit -m "Initial commit: Spectre Stealth Browser"
     git remote add origin https://github.com/USERNAME_KAMU/NAMA_REPO.git
     git push -u origin main
     ```
   - **Cara via Browser (Tanpa Git sama sekali):**
     Tinggal drag-and-drop isi folder `Spectre` ke halaman **Upload files** di repo GitHub kamu, lalu klik **Commit changes**.
3. Buka tab **Actions** di repository GitHub kamu.
4. Tunggu sekitar **3–5 menit** sampai workflow **Build Spectre Windows (.exe)** berwarna hijau ✅.
5. Download **`Spectre-Portable.exe`** dari bagian **Artifacts** (`Spectre-Windows-EXE`) atau langsung dari menu **Releases** di sisi kanan repo GitHub kamu!
