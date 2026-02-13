use slint::{SharedString, VecModel};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

slint::include_modules!();

fn main() {
    let ui = wur2::new().unwrap();

    let (tx_to_net, rx_from_ui) = mpsc::channel::<String>();
    let (tx_to_ui, rx_from_net) = mpsc::channel::<NetEvent>();

    let is_authenticated = Arc::new(AtomicBool::new(false));
    let is_authenticated_net = is_authenticated.clone();

    thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:6969").expect("failed to connect");

        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let tx_ui = tx_to_ui.clone();
        thread::spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).is_ok() {
                    let msg = line.trim().to_string();
                    if msg.starts_with("Welcome") {
                        is_authenticated_net.store(true, Ordering::Relaxed);
                    }
                    if let Some(list) = parse_user_list(&msg) {
                        let _ = tx_ui.send(NetEvent::UserList(list));
                    } else if !msg.is_empty() {
                        let _ = tx_ui.send(NetEvent::Chat(msg));
                    }
                }
            }
        });

        for msg in rx_from_ui {
            let _ = writeln!(stream, "{msg}");
        }
    });

    let tx_to_net_list = tx_to_net.clone();
    let is_authenticated_list = is_authenticated.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(3));
        if is_authenticated_list.load(Ordering::Relaxed) {
            let _ = tx_to_net_list.send("/list".to_string());
        }
    });

    let history = Rc::new(VecModel::<SharedString>::from(vec![]));
    ui.set_history(history.clone().into());

    ui.set_online_users(Rc::new(VecModel::<SharedString>::from(vec![])).into());

    let history_handle = history.clone();
    ui.on_add_to_history(move |text| {
        history_handle.push(text.clone().into());
        let msg = text.trim().to_string();
        if msg.is_empty() {
            return;
        }

        let _ = tx_to_net.send(msg);
    });

    let history_handle2 = history.clone();
    ui.on_append_message(move |msg| {
        history_handle2.push(msg.into());
    });

    let ui_weak = ui.as_weak();
    thread::spawn(move || {
        for event in rx_from_net {
            let ui_weak = ui_weak.clone();

            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    match event {
                        NetEvent::Chat(msg) => {
                            ui.invoke_append_message(msg.into());
                        }
                        NetEvent::UserList(users) => {
                            let model = Rc::new(VecModel::<SharedString>::from(
                                users
                                    .into_iter()
                                    .map(SharedString::from)
                                    .collect::<Vec<_>>(),
                            ));
                            ui.set_online_users(model.into());
                        }
                    }
                }
            })
            .unwrap();
        }
    });

    ui.run().unwrap();
}

enum NetEvent {
    Chat(String),
    UserList(Vec<String>),
}

fn parse_user_list(msg: &str) -> Option<Vec<String>> {
    let prefix = "Connected users:";
    let trimmed = msg.trim();
    if !trimmed.starts_with(prefix) {
        return None;
    }

    let list = trimmed[prefix.len()..].trim();
    if list.is_empty() {
        return Some(Vec::new());
    }

    let users = list
        .split(',')
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
        .collect::<Vec<_>>();

    Some(users)
}
