// SPDX-License-Identifier: MPL-2.0

use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget;

#[derive(Default)]
struct AboutApp {
    core: cosmic::Core,
}

#[derive(Debug, Clone)]
enum Message {
    MoreInfo,
}

impl cosmic::Application for AboutApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.yohanespradono.HelloLinux.About";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        (Self { core }, Task::none())
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::MoreInfo => {
                let _ = std::process::Command::new("cosmic-settings")
                    .arg("about")
                    .spawn();

                return cosmic::task::message(
                    cosmic::Action::Cosmic(
                        cosmic::app::Action::Close,
                    ),
                );
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        // ---- [ DATA PARSING ALA FASTFETCH ] ----
        let distro = sysinfo::System::name().unwrap_or_else(|| "Linux".into());
        let os_version = sysinfo::System::os_version().unwrap_or_else(|| "Unknown".into());
        let kernel = sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".into());
        let hostname = sysinfo::System::host_name().unwrap_or_else(|| "Unknown".into());
        let arch = std::env::consts::ARCH;

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".into());
        let cpu_cores = sys.cpus().len();
        let total_mem_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        let disks = sysinfo::Disks::new_with_refreshed_list();
        let disk_name = disks
            .first()
            .map(|d| d.name().to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".into());

        let serial = std::fs::read_to_string("/sys/class/dmi/id/product_serial")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Not available".into());

        let product_name = std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown Device".into());

        // Uptime parsing
        let uptime_secs = sysinfo::System::uptime();
        let uptime = format!("{}h {}m", uptime_secs / 3600, (uptime_secs % 3600) / 60);

        // Shell parsing
        let shell = std::env::var("SHELL")
            .map(|s| s.split('/').last().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "sh".into());

        // Desktop Environment
        let de = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_else(|_| std::env::var("DESKTOP_SESSION").unwrap_or_else(|_| "COSMIC".into()));

        // =========================
        // LEFT PANEL (Logo & Title)
        // =========================
        let left_panel = widget::column::with_capacity(3)
            .push(
                widget::icon::from_name("distributor-logo")
                    .size(128),
            )
            .push(
                widget::text::title3(format!("{} {}", distro, os_version)),
            )
            .push(
                widget::text::caption(arch),
            )
            .spacing(12)
            .align_x(Alignment::Center)
            .width(Length::Fixed(200.0));

        // =========================
        // RIGHT PANEL (Fastfetch Grid Style)
        // =========================

        // Kolom Sisi Kiri (Software & Env)
        let right_col1 = widget::column::with_capacity(6)
            .push(spec_row("OS", &distro))
            .push(spec_row("Kernel", &kernel))
            .push(spec_row("Uptime", &uptime))
            .push(spec_row("Shell", &shell))
            .push(spec_row("DE/WM", &de))
            .push(spec_row("Theme", "COSMIC-dark"))
            .spacing(8)
            .width(Length::FillPortion(1));

        // Kolom Sisi Kanan (Hardware Details)
        let right_col2 = widget::column::with_capacity(6)
            .push(spec_row("Host", &product_name))
            .push(spec_row("Hostname", &hostname))
            .push(spec_row("CPU", &cpu_brand))
            .push(spec_row("Cores", &format!("{} vCPUs", cpu_cores)))
            .push(spec_row("Memory", &format!("{total_mem_gb:.1} GB")))
            .push(spec_row("Disk", &disk_name))
            // Serial sengaja disembunyikan / ditaruh paling bawah jika dibutuhkan
            // .push(spec_row("Serial", &serial))
            .spacing(8)
            .width(Length::FillPortion(1));

        // Satukan kolom hardware dan software berdampingan
        let right_panel_grid = widget::row::with_capacity(2)
            .push(right_col1)
            .push(right_col2)
            .spacing(20)
            .width(Length::Fill);

        // Bungkus baris utama: Left Panel (Logo) + Right Panel (Grid Info)
        let info_row = widget::row::with_capacity(2)
            .push(left_panel)
            .push(right_panel_grid)
            .spacing(32)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let more_info = widget::button::standard("More Info...")
            .on_press(Message::MoreInfo);

        let footer = widget::text::caption("Hello Linux — built with libcosmic");

        // Layout vertikal menyeluruh
        let content = widget::column::with_capacity(4)
            .push(info_row)
            .push(more_info)
            .push(footer)
            .spacing(20)
            .padding(24)
            .align_x(Alignment::Center);

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .class(cosmic::theme::Container::Background)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn spec_row(label: &str, value: &str) -> Element<'static, Message> {
    use cosmic::iced::font::{Font, Weight};

    // Definisikan font tebal bawaan Iced
    let bold_font = Font {
        weight: Weight::Bold,
        ..Default::default()
    };

    widget::row::with_capacity(2)
        .push(
            widget::text::body(label.to_string())
                .font(bold_font) // Pakai .font() sebagai pengganti .bold()
                .width(Length::Fixed(75.0)),
        )
        .push(
            widget::text::body(value.to_string()),
        )
        .spacing(8)
        .into()
}

fn main() -> cosmic::iced::Result {
    // Dimensi window dilebarkan secara horizontal agar proporsional menampung dua kolom info
    let size = cosmic::iced::Size::new(860.0, 360.0);

    cosmic::app::run::<AboutApp>(
        cosmic::app::Settings::default()
            .size(size),
        (),
    )
}