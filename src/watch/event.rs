use regex;

pub struct McLogEvent {
    pub msg: String,
    pub color: String,
    pub kind: McLogEventKind,
}

pub enum McLogEventKind {
    Disconnect,
    Join,
}

pub fn get_log_event(logline :&str) -> Option<McLogEvent> {
    // todo: maybe cache those? or just compile once
    const DISCONNECT_PATTERN:&str = r"\[Server thread.*\]: (.+) lost connection: Disconnected";
    let disconnect_regex = regex::Regex::new(DISCONNECT_PATTERN).unwrap();

    const JOIN_PATTERN:&str = r"\[Server thread.*\]: (.+) joined the game";
    let join_regex = regex::Regex::new(JOIN_PATTERN).unwrap();


    if let Some(captures) = disconnect_regex.captures(logline) {
        let username = captures.get(1).unwrap();

        let msg = format!("{} has disconnected from the server. Sadge", username.as_str());
        return Some(McLogEvent{
            msg,
            color: String::from("15863310"),
            kind: McLogEventKind::Disconnect
        })
    } else if let Some(captures) = join_regex.captures(logline) {
        let username = captures.get(1).unwrap();

        let msg = format!("{} has joined server! POG", username.as_str());
        return Some(McLogEvent{
            msg,
            color: String::from("3208276"),
            kind: McLogEventKind::Join
        })
    }

    println!("DEBUG: Log line does not match any pattern.");
    return None
}
