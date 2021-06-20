
pub struct Client {
    state: State,
    reply: String,
    forward_path: String,
    reverse_path: Vec<String>,
    data: Vec<String>,
}

enum State {
    Greeted,
    SentForwardPath,
    SendingReversePaths,
    SendingData
}
