use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn menu() -> InlineKeyboardMarkup {
    let file_sharing = InlineKeyboardButton::callback("Обменник 🔁", "file_sharing");
    let my_requests = InlineKeyboardButton::callback("Мои обращения 📖", "my_requests");
    let settings = InlineKeyboardButton::callback("⚙️", "settings");
    let faq = InlineKeyboardButton::callback("FAQ ℹ️", "faq");

    InlineKeyboardMarkup::default().append_row(vec![file_sharing, my_requests]).append_row(vec![settings, faq])
}