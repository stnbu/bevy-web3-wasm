use async_channel::{unbounded, Receiver, Sender};
use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use mbutils;
use web3::transports::eip_1193;

pub struct Eip1193Plugin;

impl Plugin for Eip1193Plugin {
    fn build(&self, app: &mut App) {
        let task_pool = IoTaskPool(app.world.resource::<IoTaskPool>().0.clone());
        let (task_send, interface_receive) = unbounded();
        let (interface_send, task_receive) = unbounded();
        app.insert_resource(Eip1193Task::new(task_pool, task_send, task_receive))
            .insert_resource(Eip1193Interface::new(interface_send, interface_receive));
    }
}

pub struct Eip1193Task {
    task_pool: IoTaskPool,
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl Eip1193Task {
    pub fn new(task_pool: IoTaskPool, sender: Sender<String>, receiver: Receiver<String>) -> Self {
        Self {
            task_pool,
            sender,
            receiver,
        }
    }

    pub fn spawn(&self) {
        let provider = eip_1193::Provider::default().unwrap().unwrap();
        use web3::Transport;
        let transport = eip_1193::Eip1193::new(provider);

        let task_pool = self.task_pool.clone();
        let receiver = self.receiver.clone();
        let sender = self.sender.clone();

        let task = task_pool.spawn(async move {
            match receiver.try_recv() {
                Ok(message) => match transport.execute(&message, vec![]).await {
                    Ok(response) => {
                        match sender.try_send(response.to_string()) {
                            Ok(()) => {
                                mbutils::console_log!("Successfully sent message.")
                            }
                            Err(err) => {
                                mbutils::console_log!("Failed to send message: {}", err)
                            }
                        };
                    }
                    Err(err) => {
                        mbutils::console_log!("Failed execute web3 call: {}", err)
                    }
                },
                Err(err) => {
                    mbutils::console_log!("Failed to receive web3 api call: {}", err)
                }
            }
        });
        task.detach();
    }
}

pub struct Eip1193Interface {
    pub sender: Sender<String>,
    pub receiver: Receiver<String>,
}

impl Eip1193Interface {
    pub fn new(sender: Sender<String>, receiver: Receiver<String>) -> Self {
        Self { sender, receiver }
    }
}

fn main() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.add_system(handle_browser_resize);

    app.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_startup_system(startup)
        .add_plugin(Eip1193Plugin)
        .add_system(ui_example)
        .run();
}

fn startup(task: Res<Eip1193Task>) {
    task.spawn();
}

fn ui_example(mut egui_context: ResMut<EguiContext>, interface: Res<Eip1193Interface>) {
    egui::CentralPanel::default().show(egui_context.ctx_mut(), |ui| {
        let sender = interface.sender.clone();
        let receiver = interface.receiver.clone();
        if ui.button("metamask").clicked() {
            mbutils::console_log!("Button was clicked.");
            wasm_bindgen_futures::spawn_local(async move {
                match sender.try_send("eth_requestAccounts".to_string()) {
                    Ok(()) => {
                        mbutils::console_log!("Sent to interface sender.");
                    }
                    Err(err) => {
                        mbutils::console_log!("Not sent to interface sender: {}", err);
                    }
                };
            });
        }
        match receiver.try_recv() {
            Ok(message) => {
                let message = message.clone();
                mbutils::console_log!("A message received from interface: {}", message.to_string());
                //ui.label(message.to_string());
            }
            Err(err) => {
                mbutils::console_log!("Failed to receive from interface: {}", err);
            }
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn handle_browser_resize(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    let wasm_window = web_sys::window().unwrap();
    let (target_width, target_height) = (
        wasm_window.inner_width().unwrap().as_f64().unwrap() as f32,
        wasm_window.inner_height().unwrap().as_f64().unwrap() as f32,
    );

    if window.width() != target_width || window.height() != target_height {
        window.set_resolution(target_width, target_height);
    }
}
