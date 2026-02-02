use slint::{SharedString, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn main() {
    let ui = wur2::new().unwrap();

    let history = Rc::new(VecModel::<SharedString>::from(vec![]));
    ui.set_history(history.clone().into());

    let history_handle = history.clone();
    ui.on_add_to_history(move |text| {
        history_handle.push(text.into());
    });

    ui.run().unwrap();
}
