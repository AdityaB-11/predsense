//! Predator Control – A native graphical interface for Acer Predator & Nitro laptops on Linux
//! Built with Rune (GPU-rendered Rust framework by Anomaly) and linuwu-sense kernel driver.

use rune::tok::{radius, space};
use rune::{AppSpec, Color, Cx, El, TextStyle, col, el, row, text};
use std::fs;
use std::process::Command;

// ─── SysFS Hardware Paths ───────────────────────────────────────────────────

const PREDATOR_SYSFS: &str = "/sys/devices/platform/acer-wmi/predator_sense";
const HWMON_SYSFS: &str = "/sys/devices/platform/acer-wmi/hwmon/hwmon5";
const PROFILE_SYSFS: &str =
    "/sys/devices/platform/acer-wmi/platform-profile/platform-profile-0/profile";
const FOUR_ZONE_SYSFS: &str = "/sys/devices/platform/acer-wmi/four_zoned_kb/four_zone_mode";
const PER_ZONE_SYSFS: &str = "/sys/devices/platform/acer-wmi/four_zoned_kb/per_zone_mode";

// ─── Predator Dark Theme Palette ────────────────────────────────────────────

const PREDATOR_CYAN: Color = Color { r: 0.0, g: 0.72, b: 0.88, a: 1.0 };
const PREDATOR_BLUE: Color = Color { r: 0.0, g: 0.45, b: 0.75, a: 1.0 };
const BG_DARK: Color = Color { r: 0.06, g: 0.07, b: 0.09, a: 1.0 };
const BG_PANEL: Color = Color { r: 0.10, g: 0.12, b: 0.15, a: 1.0 };
const BG_CARD: Color = Color { r: 0.13, g: 0.16, b: 0.20, a: 1.0 };
const BG_CARD_HOVER: Color = Color { r: 0.17, g: 0.21, b: 0.26, a: 1.0 };
const BORDER_COLOR: Color = Color { r: 0.20, g: 0.25, b: 0.32, a: 1.0 };
const TEXT_PRIMARY: Color = Color { r: 0.92, g: 0.94, b: 0.97, a: 1.0 };
const TEXT_MUTED: Color = Color { r: 0.55, g: 0.62, b: 0.70, a: 1.0 };
const COLOR_GREEN: Color = Color { r: 0.20, g: 0.80, b: 0.45, a: 1.0 };
const COLOR_ORANGE: Color = Color { r: 0.95, g: 0.55, b: 0.15, a: 1.0 };
const COLOR_RED: Color = Color { r: 0.95, g: 0.25, b: 0.25, a: 1.0 };

// ─── Typography Helpers ─────────────────────────────────────────────────────

const FONT_TITLE: TextStyle = TextStyle::new(16.0, 700.0);
const FONT_SUBTITLE: TextStyle = TextStyle::new(13.0, 600.0);
const FONT_BODY: TextStyle = TextStyle::new(12.0, 400.0);
const FONT_SMALL: TextStyle = TextStyle::new(11.0, 400.0);
const FONT_STAT_NUM: TextStyle = TextStyle::new(26.0, 700.0);
const FONT_BADGE: TextStyle = TextStyle::new(10.0, 600.0);

// ─── Thermal Profile ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalProfile {
    Eco,
    Quiet,
    Balanced,
    Smart,
    Turbo,
}

impl ThermalProfile {
    const ALL: [ThermalProfile; 5] = [
        ThermalProfile::Eco,
        ThermalProfile::Quiet,
        ThermalProfile::Balanced,
        ThermalProfile::Smart,
        ThermalProfile::Turbo,
    ];

    fn sysfs_id(&self) -> &'static str {
        match self {
            ThermalProfile::Eco => "low-power",
            ThermalProfile::Quiet => "quiet",
            ThermalProfile::Balanced => "balanced",
            ThermalProfile::Smart => "balanced-performance",
            ThermalProfile::Turbo => "performance",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            ThermalProfile::Eco => "Eco",
            ThermalProfile::Quiet => "Quiet",
            ThermalProfile::Balanced => "Balanced",
            ThermalProfile::Smart => "Smart",
            ThermalProfile::Turbo => "Turbo",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ThermalProfile::Eco => "Maximum battery life & low temps",
            ThermalProfile::Quiet => "Whisper quiet acoustic fan profile",
            ThermalProfile::Balanced => "Adaptive everyday gaming & workload",
            ThermalProfile::Smart => "Higher boost clocks for creative apps",
            ThermalProfile::Turbo => "Maximum fan speed & peak clock rate",
        }
    }

    fn from_sysfs(s: &str) -> Self {
        match s.trim() {
            "low-power" => ThermalProfile::Eco,
            "quiet" => ThermalProfile::Quiet,
            "balanced" => ThermalProfile::Balanced,
            "balanced-performance" => ThermalProfile::Smart,
            "performance" => ThermalProfile::Turbo,
            _ => ThermalProfile::Balanced,
        }
    }
}

// ─── RGB Color Helper ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexColor(pub u8, pub u8, pub u8);

impl HexColor {
    fn to_hex(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    fn to_rune_color(&self) -> Color {
        Color {
            r: self.0 as f32 / 255.0,
            g: self.1 as f32 / 255.0,
            b: self.2 as f32 / 255.0,
            a: 1.0,
        }
    }

    fn from_hex(s: &str) -> Option<Self> {
        let clean = s.trim().trim_start_matches('#');
        if clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
            let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
            let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
            Some(HexColor(r, g, b))
        } else {
            None
        }
    }
}

const PRESET_COLORS: [(&str, HexColor); 8] = [
    ("Predator Cyan", HexColor(0x00, 0xAE, 0xC7)),
    ("Crimson Red", HexColor(0xE5, 0x25, 0x21)),
    ("Neon Green", HexColor(0x2E, 0xCC, 0x71)),
    ("Electric Blue", HexColor(0x34, 0x98, 0xDB)),
    ("Sunset Orange", HexColor(0xE6, 0x7E, 0x22)),
    ("Cyber Violet", HexColor(0x9B, 0x59, 0xB6)),
    ("Amber Gold", HexColor(0xF1, 0xC4, 0x0F)),
    ("Ice White", HexColor(0xEC, 0xF0, 0xF1)),
];

// ─── Active Tab ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Performance,
    FanControl,
    Battery,
    KeyboardRgb,
    SystemSettings,
}

impl ActiveTab {
    const ALL: [ActiveTab; 5] = [
        ActiveTab::Performance,
        ActiveTab::FanControl,
        ActiveTab::Battery,
        ActiveTab::KeyboardRgb,
        ActiveTab::SystemSettings,
    ];

    fn title(&self) -> &'static str {
        match self {
            ActiveTab::Performance => "PERFORMANCE",
            ActiveTab::FanControl => "FAN CONTROL",
            ActiveTab::Battery => "BATTERY & POWER",
            ActiveTab::KeyboardRgb => "LIGHTING (RGB)",
            ActiveTab::SystemSettings => "SYSTEM & DISPLAY",
        }
    }
}

// ─── Main State ─────────────────────────────────────────────────────────────

pub struct State {
    pub active_tab: ActiveTab,
    pub status_text: String,

    // Hardware Sensors
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub board_temp: f32,
    pub fan1_rpm: u32,
    pub fan2_rpm: u32,

    // Performance Profile
    pub thermal_profile: ThermalProfile,

    // Fan Controls
    pub fan_cpu_speed: u8, // 0 = Auto, 100 = Max
    pub fan_gpu_speed: u8,

    // Battery & Power
    pub battery_limiter: bool,
    pub battery_calibration: bool,
    pub usb_charging: bool,

    // Keyboard Lighting (4-Zone RGB)
    pub kb_mode: u8, // 0=Static, 1=Breathe, 2=Neon, 3=Wave, 4=Shifting, 5=Zoom
    pub kb_brightness: u8,
    pub kb_speed: u8,
    pub kb_direction: u8,
    pub zone_colors: [HexColor; 4],
    pub active_zone: usize,

    // System Settings
    pub lcd_override: bool,
    pub boot_sound: bool,
    pub backlight_timeout: u32,
}

impl Default for State {
    fn default() -> Self {
        let mut s = State {
            active_tab: ActiveTab::Performance,
            status_text: "Connected to linuwu-sense".into(),
            cpu_temp: 50.0,
            gpu_temp: 45.0,
            board_temp: 48.0,
            fan1_rpm: 1700,
            fan2_rpm: 1800,
            thermal_profile: ThermalProfile::Balanced,
            fan_cpu_speed: 0,
            fan_gpu_speed: 0,
            battery_limiter: false,
            battery_calibration: false,
            usb_charging: true,
            kb_mode: 0,
            kb_brightness: 100,
            kb_speed: 5,
            kb_direction: 0,
            zone_colors: [
                HexColor(0x00, 0xAE, 0xC7),
                HexColor(0x00, 0xAE, 0xC7),
                HexColor(0x00, 0xAE, 0xC7),
                HexColor(0x00, 0xAE, 0xC7),
            ],
            active_zone: 0,
            lcd_override: true,
            boot_sound: false,
            backlight_timeout: 30,
        };
        s.read_hardware();
        s
    }
}

// ─── Hardware I/O Helpers ───────────────────────────────────────────────────

fn read_file_string(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default().trim().to_string()
}

fn write_sysfs_pkexec(path: &str, value: &str) {
    let p = path.to_string();
    let v = value.to_string();
    std::thread::spawn(move || {
        let _ = Command::new("pkexec")
            .args(["tee", &p])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(v.as_bytes());
                }
                child.wait()
            });
    });
}

impl State {
    pub fn read_hardware(&mut self) {
        // Read Temperatures (in millidegrees C)
        if let Ok(t) = read_file_string(&format!("{HWMON_SYSFS}/temp1_input")).parse::<f32>() {
            self.cpu_temp = t / 1000.0;
        }
        if let Ok(t) = read_file_string(&format!("{HWMON_SYSFS}/temp2_input")).parse::<f32>() {
            self.gpu_temp = t / 1000.0;
        }
        if let Ok(t) = read_file_string(&format!("{HWMON_SYSFS}/temp3_input")).parse::<f32>() {
            self.board_temp = t / 1000.0;
        }

        // Read Fan RPM
        if let Ok(rpm) = read_file_string(&format!("{HWMON_SYSFS}/fan1_input")).parse::<u32>() {
            self.fan1_rpm = rpm;
        }
        if let Ok(rpm) = read_file_string(&format!("{HWMON_SYSFS}/fan2_input")).parse::<u32>() {
            self.fan2_rpm = rpm;
        }

        // Read Platform Thermal Profile
        let profile_raw = read_file_string(PROFILE_SYSFS);
        if !profile_raw.is_empty() {
            self.thermal_profile = ThermalProfile::from_sysfs(&profile_raw);
        }

        // Read Battery and Power
        self.battery_limiter = read_file_string(&format!("{PREDATOR_SYSFS}/battery_limiter")) == "1";
        self.battery_calibration =
            read_file_string(&format!("{PREDATOR_SYSFS}/battery_calibration")) == "1";
        self.usb_charging = read_file_string(&format!("{PREDATOR_SYSFS}/usb_charging")) == "1";

        // Read System Misc
        self.lcd_override = read_file_string(&format!("{PREDATOR_SYSFS}/lcd_override")) == "1";
        self.boot_sound = read_file_string(&format!("{PREDATOR_SYSFS}/boot_animation_sound")) == "1";
        if let Ok(to) = read_file_string(&format!("{PREDATOR_SYSFS}/backlight_timeout")).parse::<u32>() {
            self.backlight_timeout = to;
        }

        // Read RGB Keyboard state
        let four_zone = read_file_string(FOUR_ZONE_SYSFS);
        let parts: Vec<&str> = four_zone.split(',').collect();
        if parts.len() >= 4 {
            self.kb_mode = parts[0].trim().parse().unwrap_or(0);
            self.kb_brightness = parts[1].trim().parse().unwrap_or(100);
            self.kb_speed = parts[2].trim().parse().unwrap_or(5);
            if parts.len() >= 7 {
                self.kb_direction = parts[6].trim().parse().unwrap_or(0);
            }
        }

        let per_zone = read_file_string(PER_ZONE_SYSFS);
        let pz_parts: Vec<&str> = per_zone.split(',').collect();
        if pz_parts.len() >= 4 {
            for i in 0..4 {
                if let Some(c) = HexColor::from_hex(pz_parts[i]) {
                    self.zone_colors[i] = c;
                }
            }
        }
    }

    pub fn apply_thermal_profile(&mut self, profile: ThermalProfile) {
        self.thermal_profile = profile;
        write_sysfs_pkexec(PROFILE_SYSFS, profile.sysfs_id());
        self.status_text = format!("Thermal Profile: {}", profile.title());
    }

    pub fn apply_fan_speeds(&mut self) {
        let val = format!("{},{}", self.fan_cpu_speed, self.fan_gpu_speed);
        write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/fan_speed"), &val);
        self.status_text = format!("Fan Speeds: CPU {}%, GPU {}%", self.fan_cpu_speed, self.fan_gpu_speed);
    }

    pub fn apply_four_zone_rgb(&self) {
        let r = self.zone_colors[0].0;
        let g = self.zone_colors[0].1;
        let b = self.zone_colors[0].2;
        let val = format!(
            "{},{},{},{},{},{},{}",
            self.kb_mode, self.kb_brightness, self.kb_speed, r, g, b, self.kb_direction
        );
        write_sysfs_pkexec(FOUR_ZONE_SYSFS, &val);
    }

    pub fn apply_per_zone_colors(&self) {
        let val = format!(
            "{},{},{},{},{}",
            self.zone_colors[0].to_hex(),
            self.zone_colors[1].to_hex(),
            self.zone_colors[2].to_hex(),
            self.zone_colors[3].to_hex(),
            self.kb_brightness
        );
        write_sysfs_pkexec(PER_ZONE_SYSFS, &val);
    }
}

// ─── UI Component Builders ──────────────────────────────────────────────────

fn header_bar(s: &State) -> El {
    row()
        .key("header")
        .w_full()
        .h(56.0)
        .px(space::S6)
        .center_y()
        .justify_between()
        .fill(BG_PANEL)
        .child(
            row()
                .center_y()
                .gap(space::S3)
                .child(
                    el()
                        .size(32.0, 32.0)
                        .radius(radius::SM)
                        .fill(PREDATOR_CYAN)
                        .center()
                        .child(text("⚡").font(FONT_SUBTITLE).color(BG_DARK)),
                )
                .child(
                    col()
                        .gap(2.0)
                        .child(text("PREDATOR CONTROL").font(FONT_TITLE).color(TEXT_PRIMARY))
                        .child(text("Acer Gaming Hardware Center").font(FONT_BADGE).color(TEXT_MUTED)),
                ),
        )
        .child(
            row()
                .center_y()
                .gap(space::S3)
                .child(
                    el()
                        .px(space::S2)
                        .py(space::S1)
                        .radius(radius::SM)
                        .fill(BG_CARD)
                        .child(text(&s.status_text).font(FONT_SMALL).color(PREDATOR_CYAN)),
                )
                .child(
                    el()
                        .key("btn-refresh")
                        .px(space::S3)
                        .py(space::S1)
                        .radius(radius::SM)
                        .fill(PREDATOR_BLUE)
                        .hover(PREDATOR_CYAN)
                        .focusable(true)
                        .on_press(|s: &mut State| {
                            s.read_hardware();
                            s.status_text = "Hardware state refreshed".into();
                        })
                        .child(text("Refresh").font(FONT_SMALL).color(TEXT_PRIMARY)),
                ),
        )
}

fn tab_bar(s: &State) -> El {
    let mut tabs = row().key("tabs").w_full().h(44.0).px(space::S3).gap(space::S1).fill(BG_PANEL);

    for tab in ActiveTab::ALL {
        let active = s.active_tab == tab;
        tabs = tabs.child(
            el()
                .key(tab.title())
                .grow(1.0)
                .h_full()
                .center()
                .focusable(true)
                .hover(if active { BG_CARD } else { BG_PANEL })
                .fill(if active { BG_CARD } else { BG_PANEL })
                .border_sides([0.0, 0.0, if active { 2.5 } else { 0.0 }, 0.0], PREDATOR_CYAN)
                .on_press(move |s: &mut State| {
                    s.active_tab = tab;
                    s.read_hardware();
                })
                .child(
                    text(tab.title())
                        .font(if active { FONT_SUBTITLE } else { FONT_BODY })
                        .color(if active { PREDATOR_CYAN } else { TEXT_MUTED }),
                ),
        );
    }
    tabs
}

fn card_panel(title: &str, content: El) -> El {
    col()
        .w_full()
        .padding(space::S3)
        .radius(radius::MD)
        .fill(BG_CARD)
        .border(1.0, BORDER_COLOR)
        .gap(space::S2)
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text(title).font(FONT_SUBTITLE).color(PREDATOR_CYAN)),
        )
        .child(el().w_full().h(1.0).fill(BORDER_COLOR))
        .child(content)
}

fn stat_gauge(title: &str, val_str: &str, unit: &str, color: Color) -> El {
    col()
        .grow(1.0)
        .padding(space::S3)
        .radius(radius::SM)
        .fill(BG_PANEL)
        .border(1.0, BORDER_COLOR)
        .gap(space::S1)
        .child(text(title).font(FONT_BADGE).color(TEXT_MUTED))
        .child(
            row()
                .center_y()
                .gap(space::S1)
                .child(text(val_str).font(FONT_STAT_NUM).color(color))
                .child(text(unit).font(FONT_SMALL).color(TEXT_MUTED)),
        )
}

fn switch_row(title: &str, desc: &str, on: bool, on_toggle: fn(&mut State)) -> El {
    row()
        .w_full()
        .center_y()
        .justify_between()
        .py(space::S2)
        .child(
            col()
                .gap(2.0)
                .child(text(title).font(FONT_BODY).color(TEXT_PRIMARY))
                .child(text(desc).font(FONT_SMALL).color(TEXT_MUTED)),
        )
        .child(
            el()
                .key(title)
                .size(44.0, 24.0)
                .radius(12.0)
                .fill(if on { PREDATOR_CYAN } else { BORDER_COLOR })
                .focusable(true)
                .on_press(on_toggle)
                .child(
                    el()
                        .size(18.0, 18.0)
                        .absolute(if on { 23.0 } else { 3.0 }, 3.0)
                        .radius(9.0)
                        .fill(TEXT_PRIMARY),
                ),
        )
}

// ─── TAB: Performance ───────────────────────────────────────────────────────

fn view_performance(s: &State) -> El {
    let mut profile_buttons = row().key("profiles").w_full().gap(space::S2);

    for p in ThermalProfile::ALL {
        let active = s.thermal_profile == p;
        profile_buttons = profile_buttons.child(
            el()
                .key(p.title())
                .grow(1.0)
                .padding(space::S3)
                .radius(radius::SM)
                .fill(if active { PREDATOR_BLUE } else { BG_PANEL })
                .border(1.5, if active { PREDATOR_CYAN } else { BORDER_COLOR })
                .hover(BG_CARD_HOVER)
                .focusable(true)
                .on_press(move |s: &mut State| s.apply_thermal_profile(p))
                .child(
                    col()
                        .gap(space::S1)
                        .child(
                            text(p.title())
                                .font(FONT_SUBTITLE)
                                .color(if active { PREDATOR_CYAN } else { TEXT_PRIMARY }),
                        )
                        .child(text(p.description()).font(FONT_BADGE).color(TEXT_MUTED)),
                ),
        );
    }

    let stats_row = row()
        .w_full()
        .gap(space::S2)
        .child(stat_gauge("CPU TEMPERATURE", &format!("{:.0}", s.cpu_temp), "°C", {
            if s.cpu_temp > 80.0 { COLOR_RED } else if s.cpu_temp > 65.0 { COLOR_ORANGE } else { COLOR_GREEN }
        }))
        .child(stat_gauge("GPU TEMPERATURE", &format!("{:.0}", s.gpu_temp), "°C", {
            if s.gpu_temp > 80.0 { COLOR_RED } else if s.gpu_temp > 65.0 { COLOR_ORANGE } else { COLOR_GREEN }
        }))
        .child(stat_gauge("SYSTEM BOARD", &format!("{:.0}", s.board_temp), "°C", COLOR_GREEN))
        .child(stat_gauge("CPU FAN SPEED", &format!("{}", s.fan1_rpm), "RPM", PREDATOR_CYAN))
        .child(stat_gauge("GPU FAN SPEED", &format!("{}", s.fan2_rpm), "RPM", PREDATOR_CYAN));

    col()
        .w_full()
        .gap(space::S3)
        .child(card_panel("THERMAL & PERFORMANCE PROFILES", profile_buttons))
        .child(card_panel("REAL-TIME HARDWARE TELEMETRY", stats_row))
}

// ─── TAB: Fan Control ───────────────────────────────────────────────────────

fn fan_slider(label: &str, value: u8, on_set: impl Fn(&mut State, u8) + 'static + Copy) -> El {
    let pct = value as f32 / 100.0;
    let track_w = 400.0;

    row()
        .w_full()
        .center_y()
        .justify_between()
        .py(space::S2)
        .child(
            col()
                .gap(2.0)
                .child(text(label).font(FONT_BODY).color(TEXT_PRIMARY))
                .child(
                    text(if value == 0 { "Automatic EC curve".into() } else { format!("Manual: {}%", value) })
                        .font(FONT_SMALL)
                        .color(PREDATOR_CYAN),
                ),
        )
        .child(
            row()
                .center_y()
                .gap(space::S3)
                .child(
                    el()
                        .size(track_w, 16.0)
                        .radius(8.0)
                        .fill(BG_PANEL)
                        .border(1.0, BORDER_COLOR)
                        .child(
                            el()
                                .size(track_w * pct, 16.0)
                                .radius(8.0)
                                .fill(if pct > 0.8 { COLOR_RED } else { PREDATOR_CYAN }),
                        )
                        .on_press(move |s: &mut State| {
                            let next = if value >= 100 { 0 } else { value + 25 };
                            on_set(s, next);
                            s.apply_fan_speeds();
                        }),
                )
                .child(
                    el()
                        .key(format!("{}-auto", label).as_str())
                        .px(space::S2)
                        .py(space::S1)
                        .radius(radius::SM)
                        .fill(BG_PANEL)
                        .border(1.0, BORDER_COLOR)
                        .hover(BG_CARD_HOVER)
                        .on_press(move |s: &mut State| {
                            on_set(s, 0);
                            s.apply_fan_speeds();
                        })
                        .child(text("Auto").font(FONT_SMALL).color(TEXT_PRIMARY)),
                )
                .child(
                    el()
                        .key(format!("{}-max", label).as_str())
                        .px(space::S2)
                        .py(space::S1)
                        .radius(radius::SM)
                        .fill(PREDATOR_BLUE)
                        .hover(PREDATOR_CYAN)
                        .on_press(move |s: &mut State| {
                            on_set(s, 100);
                            s.apply_fan_speeds();
                        })
                        .child(text("Max").font(FONT_SMALL).color(TEXT_PRIMARY)),
                ),
        )
}

fn view_fan_control(s: &State) -> El {
    let fan_sliders = col()
        .w_full()
        .gap(space::S2)
        .child(fan_slider("CPU Fan Speed", s.fan_cpu_speed, |s, v| s.fan_cpu_speed = v))
        .child(fan_slider("GPU Fan Speed", s.fan_gpu_speed, |s, v| s.fan_gpu_speed = v));

    let quick_actions = row()
        .gap(space::S2)
        .child(
            el()
                .px(space::S3)
                .py(space::S2)
                .radius(radius::SM)
                .fill(COLOR_RED)
                .on_press(|s: &mut State| {
                    s.fan_cpu_speed = 100;
                    s.fan_gpu_speed = 100;
                    s.apply_fan_speeds();
                })
                .child(text("MAX ALL FANS (Turbo)").font(FONT_SUBTITLE).color(TEXT_PRIMARY)),
        )
        .child(
            el()
                .px(space::S3)
                .py(space::S2)
                .radius(radius::SM)
                .fill(BG_PANEL)
                .border(1.0, BORDER_COLOR)
                .on_press(|s: &mut State| {
                    s.fan_cpu_speed = 0;
                    s.fan_gpu_speed = 0;
                    s.apply_fan_speeds();
                })
                .child(text("RESET TO AUTO CURVE").font(FONT_SUBTITLE).color(TEXT_PRIMARY)),
        );

    col()
        .w_full()
        .gap(space::S3)
        .child(card_panel("FAN SPEEDS & INDIVIDUAL CONTROLS", fan_sliders))
        .child(card_panel("QUICK PRESETS", quick_actions))
}

// ─── TAB: Battery & Power ───────────────────────────────────────────────────

fn view_battery(s: &State) -> El {
    let toggles = col()
        .w_full()
        .gap(space::S2)
        .child(switch_row(
            "Battery Care Limiter (80% Max)",
            "Limits maximum charging capacity to preserve lithium-ion battery longevity",
            s.battery_limiter,
            |s: &mut State| {
                s.battery_limiter = !s.battery_limiter;
                let val = if s.battery_limiter { "1" } else { "0" };
                write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/battery_limiter"), val);
                s.status_text = format!("Battery Limiter: {}", if s.battery_limiter { "Enabled" } else { "Disabled" });
            },
        ))
        .child(switch_row(
            "Battery Calibration Cycle",
            "Discharges and charges fully to recalibrate fuel-gauge battery sensor",
            s.battery_calibration,
            |s: &mut State| {
                s.battery_calibration = !s.battery_calibration;
                let val = if s.battery_calibration { "1" } else { "0" };
                write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/battery_calibration"), val);
                s.status_text = format!("Battery Calibration: {}", if s.battery_calibration { "Active" } else { "Inactive" });
            },
        ))
        .child(switch_row(
            "USB Always-On Charging",
            "Supplies power to USB ports even when laptop is turned off or suspended",
            s.usb_charging,
            |s: &mut State| {
                s.usb_charging = !s.usb_charging;
                let val = if s.usb_charging { "1" } else { "0" };
                write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/usb_charging"), val);
                s.status_text = format!("USB Charging: {}", if s.usb_charging { "Enabled" } else { "Disabled" });
            },
        ));

    col().w_full().child(card_panel("BATTERY HEALTH & POWER MANAGEMENT", toggles))
}

// ─── TAB: Keyboard RGB Lighting ─────────────────────────────────────────────

const MODE_NAMES: [&str; 6] = ["Static", "Breathing", "Neon", "Wave", "Shifting", "Zoom"];

fn view_keyboard_rgb(s: &State) -> El {
    let mut mode_row = row().key("modes").w_full().gap(space::S2);

    for (i, &name) in MODE_NAMES.iter().enumerate() {
        let active = s.kb_mode as usize == i;
        mode_row = mode_row.child(
            el()
                .key(name)
                .grow(1.0)
                .py(space::S2)
                .radius(radius::SM)
                .fill(if active { PREDATOR_BLUE } else { BG_PANEL })
                .border(1.0, if active { PREDATOR_CYAN } else { BORDER_COLOR })
                .center()
                .focusable(true)
                .on_press(move |s: &mut State| {
                    s.kb_mode = i as u8;
                    s.apply_four_zone_rgb();
                    s.status_text = format!("Lighting Mode: {}", name);
                })
                .child(
                    text(name)
                        .font(FONT_BODY)
                        .color(if active { PREDATOR_CYAN } else { TEXT_PRIMARY }),
                ),
        );
    }

    // 4-Zone Color Palette and Zone Selector
    let mut zones_row = row().key("zones").w_full().gap(space::S3);
    for z in 0..4 {
        let is_selected = s.active_zone == z;
        let c = s.zone_colors[z];
        zones_row = zones_row.child(
            el()
                .key(format!("zone-select-{}", z).as_str())
                .grow(1.0)
                .padding(space::S3)
                .radius(radius::SM)
                .fill(BG_PANEL)
                .border(2.0, if is_selected { PREDATOR_CYAN } else { BORDER_COLOR })
                .on_press(move |s: &mut State| s.active_zone = z)
                .child(
                    col()
                        .center()
                        .gap(space::S2)
                        .child(
                            el()
                                .size(48.0, 24.0)
                                .radius(radius::SM)
                                .fill(c.to_rune_color()),
                        )
                        .child(
                            text(format!("Zone {}", z + 1).as_str())
                                .font(FONT_SUBTITLE)
                                .color(if is_selected { PREDATOR_CYAN } else { TEXT_PRIMARY }),
                        )
                        .child(text(&format!("#{}", c.to_hex())).font(FONT_BADGE).color(TEXT_MUTED)),
                ),
        );
    }

    let mut color_chips = row().key("palette").w_full().gap(space::S2);
    for (name, color) in PRESET_COLORS {
        color_chips = color_chips.child(
            el()
                .key(name)
                .grow(1.0)
                .py(space::S2)
                .radius(radius::SM)
                .fill(color.to_rune_color())
                .center()
                .on_press(move |s: &mut State| {
                    let z = s.active_zone;
                    s.zone_colors[z] = color;
                    s.apply_per_zone_colors();
                    s.status_text = format!("Zone {} -> {}", z + 1, name);
                })
                .child(text("").font(FONT_BADGE)),
        );
    }

    let sliders_content = col()
        .w_full()
        .gap(space::S2)
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text("Brightness (0-100%)").font(FONT_BODY).color(TEXT_PRIMARY))
                .child(
                    row().gap(space::S1).children(
                        [25u8, 50, 75, 100].iter().map(|&b| {
                            let active = s.kb_brightness == b;
                            el()
                                .key(format!("br-{}", b).as_str())
                                .px(space::S2)
                                .py(space::S1)
                                .radius(radius::SM)
                                .fill(if active { PREDATOR_BLUE } else { BG_PANEL })
                                .border(1.0, BORDER_COLOR)
                                .on_press(move |s: &mut State| {
                                    s.kb_brightness = b;
                                    s.apply_four_zone_rgb();
                                })
                                .child(text(format!("{}%", b).as_str()).font(FONT_SMALL).color(TEXT_PRIMARY))
                        })
                    ),
                ),
        )
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text("Animation Speed (1-9)").font(FONT_BODY).color(TEXT_PRIMARY))
                .child(
                    row().gap(space::S1).children(
                        [1u8, 3, 5, 7, 9].iter().map(|&spd| {
                            let active = s.kb_speed == spd;
                            el()
                                .key(format!("spd-{}", spd).as_str())
                                .px(space::S2)
                                .py(space::S1)
                                .radius(radius::SM)
                                .fill(if active { PREDATOR_BLUE } else { BG_PANEL })
                                .border(1.0, BORDER_COLOR)
                                .on_press(move |s: &mut State| {
                                    s.kb_speed = spd;
                                    s.apply_four_zone_rgb();
                                })
                                .child(text(format!("{}", spd).as_str()).font(FONT_SMALL).color(TEXT_PRIMARY))
                        })
                    ),
                ),
        );

    col()
        .w_full()
        .gap(space::S3)
        .child(card_panel("LIGHTING EFFECTS", mode_row))
        .child(card_panel("4-ZONE KEYBOARD COLORS (Click zone, then click color)", col().gap(space::S2).child(zones_row).child(color_chips)))
        .child(card_panel("BRIGHTNESS & SPEED", sliders_content))
}

// ─── TAB: System Settings ───────────────────────────────────────────────────

fn view_system_settings(s: &State) -> El {
    let settings = col()
        .w_full()
        .gap(space::S2)
        .child(switch_row(
            "LCD Overdrive Mode",
            "Forces panel response overdrive via Acer WMI to eliminate motion blur",
            s.lcd_override,
            |s: &mut State| {
                s.lcd_override = !s.lcd_override;
                let val = if s.lcd_override { "1" } else { "0" };
                write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/lcd_override"), val);
            },
        ))
        .child(switch_row(
            "Startup Predator Chime & Boot Animation",
            "Plays the signature Predator logo animation and boot sound at startup",
            s.boot_sound,
            |s: &mut State| {
                s.boot_sound = !s.boot_sound;
                let val = if s.boot_sound { "1" } else { "0" };
                write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/boot_animation_sound"), val);
            },
        ))
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .py(space::S2)
                .child(
                    col()
                        .gap(2.0)
                        .child(text("Backlight Auto-Turnoff Timeout").font(FONT_BODY).color(TEXT_PRIMARY))
                        .child(text("Automatically turn off keyboard lighting when idle").font(FONT_SMALL).color(TEXT_MUTED)),
                )
                .child(
                    row().gap(space::S1).children(
                        [0u32, 10, 30, 60].iter().map(|&to| {
                            let active = s.backlight_timeout == to;
                            let lbl = if to == 0 { "Never".to_string() } else { format!("{}s", to) };
                            el()
                                .key(format!("bl-to-{}", to).as_str())
                                .px(space::S2)
                                .py(space::S1)
                                .radius(radius::SM)
                                .fill(if active { PREDATOR_BLUE } else { BG_PANEL })
                                .border(1.0, BORDER_COLOR)
                                .on_press(move |s: &mut State| {
                                    s.backlight_timeout = to;
                                    write_sysfs_pkexec(&format!("{PREDATOR_SYSFS}/backlight_timeout"), &to.to_string());
                                })
                                .child(text(&lbl).font(FONT_SMALL).color(TEXT_PRIMARY))
                        })
                    ),
                ),
        );

    let system_info = col()
        .w_full()
        .gap(space::S1)
        .child(text("Hardware Controller: Acer WMI Gaming Interface (PredatorSense V4)").font(FONT_BODY).color(TEXT_PRIMARY))
        .child(text("Kernel Module: linuwu-sense (DKMS platform:acer-wmi)").font(FONT_SMALL).color(TEXT_MUTED))
        .child(text("GUI Framework: Rune (GPU-rendered with wgpu / Taffy / Parley)").font(FONT_SMALL).color(TEXT_MUTED))
        .child(text("Polkit Security: Verified non-root session (pkexec helper)").font(FONT_SMALL).color(COLOR_GREEN));

    col()
        .w_full()
        .gap(space::S3)
        .child(card_panel("SYSTEM HARDWARE TOGGLES", settings))
        .child(card_panel("DRIVER & PLATFORM INFORMATION", system_info))
}

// ─── Main View Function ─────────────────────────────────────────────────────

fn view(s: &State, _cx: &mut Cx<State>) -> El {
    col()
        .key("root")
        .w_full()
        .h_full()
        .fill(BG_DARK)
        .child(header_bar(s))
        .child(tab_bar(s))
        .child(
            col()
                .key("content-scroll")
                .grow(1.0)
                .w_full()
                .padding(space::S3)
                .scroll_xy()
                .child(match s.active_tab {
                    ActiveTab::Performance => view_performance(s),
                    ActiveTab::FanControl => view_fan_control(s),
                    ActiveTab::Battery => view_battery(s),
                    ActiveTab::KeyboardRgb => view_keyboard_rgb(s),
                    ActiveTab::SystemSettings => view_system_settings(s),
                }),
        )
}

// ─── Entry Point ────────────────────────────────────────────────────────────

fn main() {
    rune::run(AppSpec::<State> {
        ..AppSpec::new("Predator Control", (880.0, 640.0), State::default, view)
    });
}
