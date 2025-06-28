use db_pg::Message;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn history() -> InlineKeyboardMarkup {
    let all = InlineKeyboardButton::callback("Все собщения", "all_requests");
    let answered = InlineKeyboardButton::callback("С ответом", "answered_requests");
    let accepted = InlineKeyboardButton::callback("Принятые", "accepted_requests");
    let back_to_menu = InlineKeyboardButton::callback("⬅️", "back_to_menu");

    InlineKeyboardMarkup::default().append_row(vec![all]).append_row(vec![answered, accepted])
}

pub const ITEMS_PER_PAGE: usize = 10;

pub fn all_messages(messages: Vec<Message>, current_page: usize) -> InlineKeyboardMarkup {
    let total_pages = (messages.len().max(1) + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
    let current_page = current_page.min(total_pages.saturating_sub(1));

    let start_idx = current_page * ITEMS_PER_PAGE;
    let end_idx = (start_idx + ITEMS_PER_PAGE).min(messages.len());

    let mut rows = messages[start_idx..end_idx]
        .iter()
        .map(|msg| {
            let text = msg.text.chars().take(10).collect::<String>();
            let button_text = if msg.text.chars().count() > 10 {
                format!("{}...", text)
            } else {
                text
            };
            vec![InlineKeyboardButton::callback(
                button_text,
                format!("msg_{}", msg.id)
            )]
        })
        .collect::<Vec<_>>();

    // Добавляем панель навигации, если нужно
    if total_pages > 1 {
        let navigation_row = create_navigation_row(current_page, total_pages);
        rows.push(navigation_row);
    }

    InlineKeyboardMarkup::new(rows)
}

pub fn create_navigation_row(current_page: usize, total_pages: usize) -> Vec<InlineKeyboardButton> {
    let mut buttons = Vec::new();
    
    // Кнопка "Назад"
    if current_page > 0 {
        buttons.push(InlineKeyboardButton::callback(
            "⬅️",
            format!("page_{}", current_page.saturating_sub(1))
        ));
    }
    
    // Кнопка с текущей страницей
    buttons.push(InlineKeyboardButton::callback(
        format!("{}/{}", current_page + 1, total_pages),
        "current_page".to_string()
    ));
    
    // Кнопка "Вперед"
    if current_page < total_pages - 1 {
        buttons.push(InlineKeyboardButton::callback(
            "➡️",
            format!("page_{}", current_page + 1)
        ));
    }
    
    buttons
}