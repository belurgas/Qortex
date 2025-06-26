#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    OnWaiting,
    Send,
    WaitQuestion,
}