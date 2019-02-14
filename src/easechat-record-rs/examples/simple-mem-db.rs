use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use chrono::{DateTime, Local};

fn main() {
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

    conn.execute(
        "INSERT INTO msg (time, channel, text) values (?1, ?2, ?3)",
        &[&Local::now() as &dyn ToSql, &"c:123", &"d:456"]
    ).unwrap();

    let mut stmt = conn
        .prepare("SELECT id, time, channel, text FROM msg")
        .unwrap();
    let mut iter = stmt
        .query(NO_PARAMS)
        .unwrap();

    while let Some(Ok(row)) = iter.next() {
        let id: u32 = row.get("id");
        let time: DateTime<Local> = row.get("time");
        let channel: String = row.get("channel");
        let text: String = row.get("text");
        println!("Found row {:?}", (id, time, channel, text));
    }

    /*
      
    let conn = Connection::open_in_memory().unwrap();

    conn.execute(
        "CREATE TABLE person (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  time_created    TEXT NOT NULL,
                  data            BLOB
                  )",
        NO_PARAMS,
    ).unwrap();
    let me = Person {
        id: 0,
        name: "Steven".to_string(),
        time_created: time::get_time(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, time_created, data)
                  VALUES (?1, ?2, ?3)",
        &[&me.name as &ToSql, &me.time_created, &me.data],
    ).unwrap();

    let mut stmt = conn
        .prepare("SELECT id, name, time_created, data FROM person")
        .unwrap();
    let person_iter = stmt
        .query_map(NO_PARAMS, |row| Person {
            id: row.get(0),
            name: row.get(1),
            time_created: row.get(2),
            data: row.get(3),
        }).unwrap();

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    */
}