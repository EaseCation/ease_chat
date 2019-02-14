#[macro_use]
extern crate clap;

use std::thread;
use rusqlite::{NO_PARAMS, Connection};
use clap::App;

fn main() { 
    let matches = clap_app!(easechat_record =>
        (version: "0.0.0")
        (author: "Luo Jia <me@luojia.cc>")
        (about: "Connect to easechat server and record messages on some channels")
        (@arg HOSTNAME: +required "Sets the host ip address of server")
        (@arg PORT: -p --port +takes_value "Sets the port of server, default to 6500")
        (@arg SQLITE_PATH: --save-sqlite +takes_value "Sets a SQLite file path for storage")
    ).get_matches();
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
