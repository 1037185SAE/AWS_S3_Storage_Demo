#![allow(unused)]

fn dialog(level: native_dialog::MessageLevel, title: &str, msg: &str) {
    native_dialog::DialogBuilder::message()
        .set_level(level)
        .set_title(title)
        .set_text(msg)
        .alert()
        .show()
        .unwrap();
}

pub(crate) fn dialog_info(title: &str, msg: &str) {
    let title = if title != "" { title } else { "Info" };

    dialog(native_dialog::MessageLevel::Info, title, msg);
}

pub(crate) fn dialog_warning(title: &str, msg: &str) {
    let title = if title != "" { title } else { "Warning" };

    dialog(native_dialog::MessageLevel::Warning, title, msg);
}

pub(crate) fn dialog_error(title: &str, msg: &str) {
    let title = if title != "" { title } else { "Error" };

    dialog(native_dialog::MessageLevel::Error, title, msg);
}