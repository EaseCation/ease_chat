use ease_chat_nexus::parse_console_command;

fn main() {
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        match parse_console_command(&buffer) {
            Ok(Some(command)) => println!("{:?}", command),
            Ok(None) => {}
            Err(error) => eprintln!("err: <Console Input> {}", error),
        }
    }
}
