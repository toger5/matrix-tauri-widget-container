use std::{env, process::exit};

pub fn get_args() -> (String, String, String){
    match (env::args().nth(1), env::args().nth(2), env::args().nth(3)) {
        (Some(a), Some(b), Some(c)) => (a, b, c),
        _ => {
            eprintln!(
                "Usage: {} <homeserver_url> <username> <password>",
                env::args().next().unwrap()
            );
            // exist if missing
            exit(1)
        }
    }
}
