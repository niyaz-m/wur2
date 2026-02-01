slint::include_modules!();

fn main() {
    let ui = wur2::new().unwrap();

    ui.on_send_message(|text| {
        println!("User entered: {}", text);
    });

    ui.run().unwrap();
}
