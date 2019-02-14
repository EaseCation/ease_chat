use std::thread;
use rusqlite::{NO_PARAMS, Connection};
use clap::{App, load_yaml};

fn main() { 
    let yaml = load_yaml!("recorder-cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS msg (
            id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            time            TEXT NOT NULL,
            channel         TEXT NOT NULL,
            text            TEXT NOT NULL
        )",
        NO_PARAMS,
    ).unwrap();

    // let url = "localhost:6500";

    // thread::spawn(move || {
    //     ws::connect(url, |out| {
    //         out.send("Hello WebSocket").unwrap();
    //         move |msg| {
    //             println!("Got message: {}", msg);
    //             out.close(CloseCode::Normal)
    //         }
    //     }).unwrap();
    // });
}
