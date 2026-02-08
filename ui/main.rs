use slint::{SharedString, VecModel};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

slint::include_modules!();

fn main() {
    let ui = wur2::new().unwrap();

    let (tx_to_net, rx_from_ui) = mpsc::channel::<String>();
    let (tx_to_ui, rx_from_net) = mpsc::channel::<String>();

    thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:6969").expect("failed to connect");

        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let tx_ui = tx_to_ui.clone();
        thread::spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).is_ok() {
                    let _ = tx_ui.send(line.trim().to_string());
                }
            }
        });

        for msg in rx_from_ui {
            let _ = writeln!(stream, "{msg}");
        }
    });

    let history = Rc::new(VecModel::<SharedString>::from(vec![]));
    ui.set_history(history.clone().into());

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
        for msg in rx_from_net {
            let ui_weak = ui_weak.clone();
            let msg = msg.clone();

            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.invoke_append_message(msg.into());
                }
            }).unwrap();
        }
    });

    ui.run().unwrap();
}
