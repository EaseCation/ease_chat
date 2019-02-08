use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use chrono::{DateTime, Local};

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    time_created: DateTime<Local>,
    data: Option<Vec<u8>>,
}

fn main() {
    let conn = Connection::open_in_memory().unwrap();

    conn.execute(
        "CREATE TABLE person (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  time_created    TEXT NOT NULL,
                  data            BLOB
                  )",
        NO_PARAMS,
    )
    .unwrap();
    let me = Person {
        id: 0,
        name: "Steven".to_string(),
        time_created: Local::now(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, time_created, data)
                  VALUES (?1, ?2, ?3)",
        &[&me.name as &dyn ToSql, &me.time_created, &me.data],
    )
    .unwrap();

    let mut stmt = conn
        .prepare("SELECT id, name, time_created, data FROM person")
        .unwrap();
    let person_iter = stmt
        .query_map(NO_PARAMS, |row| Person {
            id: row.get(0),
            name: row.get(1),
            time_created: row.get(2),
            data: row.get(3),
        })
        .unwrap();

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
}