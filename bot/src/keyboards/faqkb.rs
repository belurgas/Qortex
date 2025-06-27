use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn faq() -> InlineKeyboardMarkup {
    let q1 = InlineKeyboardButton::callback("🤔 Чем полезен этот бот?", "profits");
    
    // Необходимо если вызывается из Inline menu
    let back_to_faq = InlineKeyboardButton::callback("⬅️", "back_to_menu");
    
    InlineKeyboardMarkup::default().append_row(vec![q1]).append_row(vec![back_to_faq])
}

pub fn feedback_ai() -> InlineKeyboardMarkup {
    let yes = InlineKeyboardButton::callback("✅", "fb_yes");
    let no  = InlineKeyboardButton::callback("⛔", "fb_no");
    let back_to_faq = InlineKeyboardButton::callback("⬅️", "back_to_faq");

    InlineKeyboardMarkup::default().append_row(vec![yes, no]).append_row(vec![back_to_faq])
}

pub fn profits() -> InlineKeyboardMarkup {
    let back_to_faq = InlineKeyboardButton::callback("⬅️", "back_to_faq");
    // Ещё вопросы если надо

    InlineKeyboardMarkup::default().append_row(vec![back_to_faq])
}