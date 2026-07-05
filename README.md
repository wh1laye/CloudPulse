# ☁️ CloudPulse

Красивый системный монитор с прозрачным glass-интерфейсом для Windows и Linux.

## Что умеет
- 📊 **CPU** — загрузка, название, количество ядер
- 🎮 **GPU** — загрузка, VRAM, температура (NVIDIA / AMD)
- 💾 **RAM** — использование памяти
- 💿 **Disk** — использование диска
- 🌐 **Network** — скорость download/upload в реальном времени
- 💻 **System Info** — hostname, OS, kernel, uptime
- 📈 **Top Processes** — топ-5 процессов по CPU
- 🪟 **Glass UI** — прозрачный интерфейс с blur-эффектом
- ⚡ **Быстрый** — написан на Rust + Tauri

## Как скачать
Перейди в раздел [Releases](https://github.com/wh1laye/CloudPulse/releases) и скачай установщик.

## Как собрать самому
```bash
git clone https://github.com/wh1laye/CloudPulse.git
cd CloudPulse
npm install
cargo tauri dev
GPU мониторинг
    NVIDIA: через nvidia-smi
    AMD (Linux): через sysfs /sys/class/drm/ (без установки драйверов)
    AMD (Windows): через rocm-smi (если установлен)

Лицензия
MIT — свободное использование.
Автор
wh1laye
