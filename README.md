# Predator Control

A **PredatorSense-style graphical control panel** for Acer Predator gaming laptops on Linux, built with the [Rune](https://github.com/anomalyco/rune) Rust UI framework on top of the [linuwu-sense](https://github.com/0x7375646F/Linuwu-Sense) kernel module.

---

## Features

| Tab | Controls |
|-----|----------|
| **Performance** | Thermal profiles (Eco / Quiet / Balanced / Smart / Turbo), live CPU/GPU/board temps, live fan RPM |
| **Fan** | Manual CPU & GPU fan speed sliders (0 = auto), live RPM readout |
| **Battery** | Battery limiter (80 % cap), battery calibration, USB always-on charging |
| **Keyboard** | Lighting mode (Static / Breathe / Neon / Wave / Shifting / Zoom), per-zone colour swatches, brightness & speed sliders, wave direction |
| **System** | LCD override, boot animation & sound, keyboard backlight timeout, about |

All writes go through **polkit** (`pkexec tee …`) — no setuid binary, no permanent sudo.

---

## Prerequisites

- Arch Linux (or any systemd distro)
- `linuwu-sense` DKMS module loaded (`lsmod | grep linuwu_sense`)
- Rust stable (`rustup install stable`)
- `polkit` package

---

## Build & Install

```bash
cd ~/Projects/current/Predator_control
bash install.sh
```

Or manually:

```bash
cargo build --release
sudo install -Dm755 target/release/predator-control /usr/local/bin/predator-control
sudo install -Dm644 policy/com.aditya.predator-control.policy \
    /usr/share/polkit-1/actions/com.aditya.predator-control.policy
sudo install -Dm644 assets/predator-control.desktop \
    /usr/share/applications/predator-control.desktop
```

---

## Running

```bash
predator-control
# or from app launcher: "Predator Control"
```

On first write operation, polkit will ask for your password. The session key is kept for the duration of your desktop session.

---

## Sysfs paths used

| Feature | Path |
|---------|------|
| Thermal profile | `/sys/devices/platform/acer-wmi/platform-profile/platform-profile-0/profile` |
| Fan speed (R/W) | `/sys/devices/platform/acer-wmi/predator_sense/fan_speed` |
| Fan 1 RPM | `/sys/devices/platform/acer-wmi/hwmon/hwmon5/fan1_input` |
| Fan 2 RPM | `/sys/devices/platform/acer-wmi/hwmon/hwmon5/fan2_input` |
| CPU temp | `/sys/devices/platform/acer-wmi/hwmon/hwmon5/temp1_input` |
| GPU temp | `/sys/devices/platform/acer-wmi/hwmon/hwmon5/temp2_input` |
| Board temp | `/sys/devices/platform/acer-wmi/hwmon/hwmon5/temp3_input` |
| Battery limiter | `/sys/devices/platform/acer-wmi/predator_sense/battery_limiter` |
| Battery calibration | `/sys/devices/platform/acer-wmi/predator_sense/battery_calibration` |
| USB charging | `/sys/devices/platform/acer-wmi/predator_sense/usb_charging` |
| LCD override | `/sys/devices/platform/acer-wmi/predator_sense/lcd_override` |
| Boot anim+sound | `/sys/devices/platform/acer-wmi/predator_sense/boot_animation_sound` |
| Backlight timeout | `/sys/devices/platform/acer-wmi/predator_sense/backlight_timeout` |
| KB 4-zone mode | `/sys/devices/platform/acer-wmi/four_zoned_kb/four_zone_mode` |
| KB per-zone colour | `/sys/devices/platform/acer-wmi/four_zoned_kb/per_zone_mode` |

---

## Architecture

```
src/main.rs
├── State          – single source of truth for all hardware & UI state
├── poll_sensors() – async loop (every 2 s) reading hwmon values
├── load_initial_state() – reads all sysfs attributes once on startup
├── sysfs_write()  – pkexec tee wrapper (non-blocking tokio spawn)
├── view()         – pure Rune render function
│   ├── title bar
│   ├── tab bar   – Performance / Fan / Battery / Keyboard / System
│   └── tab_*()   – per-tab content builders
└── main()         – Tokio runtime + Rune Runtime
```

---

## Licence

MIT
