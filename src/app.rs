// SPDX-License-Identifier: MPL-2.0

use std::process::Command;
use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::widget::column;
use cosmic::iced::{futures, window::Id, Left, Length, Limits, Subscription};
use cosmic::prelude::*;
use futures::SinkExt;
use cosmic::applet::{menu_button};
use cosmic::widget::{self, Container};

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    About,
    Settings,
    Terminal,
    Files,
    Sleep,
    Restart,
    Shutdown,
    LockScreen,
    Logout,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.yohanespradono.HelloCosmicLinux";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {

        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }
    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("distributor-logo")
            .on_press_down(Message::TogglePopup)
            .padding(0)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let item = |label: &'static str, msg: Message| {
            menu_button(widget::text::body(label))
                .on_press(msg)
                .width(Length::Fill) // Extends the clickable area fully to 240
                .into()
        };

        let divider = || widget::container(widget::divider::horizontal::default())
            .padding([4, 0]);

        let container = widget::container(
            column(vec![
                item("About This Linux", Message::About),
                item("Settings", Message::Settings),
                divider().into(),
                item("Terminal", Message::Terminal),
                item("Files", Message::Files),
                divider().into(),
                item("Sleep", Message::Sleep),
                item("Restart", Message::Restart),
                item("Shutdown", Message::Shutdown),
                divider().into(),
                item("Lock Screen", Message::LockScreen),
                item("Log Out", Message::Logout),
            ])
                .padding([1,1])
                //.width(Length::Fixed(240.0))
        )
            .padding(6);

        self.core
            .applet
            .popup_container(container)
            .limits(
                Limits::NONE
                    .min_width(1.)
                    .min_height(1.)
                    .max_width(250.)   // <-- ini
                    .max_height(1000.),
            )
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            Subscription::run(|| {
                cosmic::iced::stream::channel(4, move |mut channel: futures::channel::mpsc::Sender<_>| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;
                    futures::future::pending().await
                })
            }),
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SubscriptionChannel => {}
            Message::UpdateConfig(config) => self.config = config,

            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup = Some(new_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        Some((240, 500)), // Change the initial surface width hint to 240 here
                        None,
                        None,
                    );

                    // Force the underlying Wayland Layer Surface window limits down to 240
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(240.0)
                        .min_width(240.0)
                        .max_height(500.0)
                        .min_height(200.0);

                    get_popup(popup_settings)
                };
            }

            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }

            Message::About => {
                if let Ok(exe) = std::env::current_exe() {
                    if let Some(dir) = exe.parent() {
                        match Command::new(dir.join("hello-linux-about")).spawn() {
                            Ok(_) => {}
                            Err(e) => eprintln!("gagal buka about dialog: {e}"),
                        }
                    }
                }
            }

            Message::Settings => {
                let _ = Command::new("cosmic-settings")
                    .arg("about")
                    .spawn();
            },

            Message::Sleep => println!("Sleep clicked"),
            Message::Restart => println!("Restart clicked"),
            Message::Shutdown => println!("Shutdown clicked"),
            Message::LockScreen => println!("Lock Screen clicked"),
            Message::Logout => println!("Logout clicked"),
            Message::Terminal => println!("Terminal"),
            Message::Files => println!("Files"),
        }

        Task::none()
    }
}
