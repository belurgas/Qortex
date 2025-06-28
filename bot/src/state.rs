use db_pg::Message;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    OnWaiting,
    Send,
    WaitQuestion,
    ViewingMessages {
        messages: Vec<Message>,
        current_page: usize,
    },
    ViewingSingleMessage {
        message: Message,
        back_page: usize,
    },
}