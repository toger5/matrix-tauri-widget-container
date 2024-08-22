use std::{env, process::exit};

pub struct Args {
    pub homeserver_url: String,
    pub username: String,
    pub password: String,
    pub room_id: String,
}
pub fn get_args() -> Args {
    match (
        env::args().nth(1),
        env::args().nth(2),
        env::args().nth(3),
        env::args().nth(4),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => Args {
            homeserver_url: a,
            username: b,
            password: c,
            room_id: d,
        },
        _ => {
            eprintln!(
                "Usage: {} <homeserver_url> <username> <password> <room_id>",
                env::args().next().unwrap()
            );
            // exit if missing
            exit(1)
        }
    }
}
