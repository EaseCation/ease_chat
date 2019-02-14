use pest_derive::Parser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "../src/console_rule.pest"]
struct NexusParser;

fn main() {
    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        match NexusParser::parse(Rule::command, &buf.trim()) {
            Ok(mut pairs) => match pairs.next().map(|p| p.as_rule()) {
                Some(Rule::cmd_quit_head) => std::process::exit(0),
                Some(Rule::cmd_list_head) => println!("list!!"),
                Some(Rule::cmd_push_head) => if let (Some(sender), Some(chan), Some(msg)) 
                    = (pairs.next(), pairs.next(), pairs.next()) {
                    println!("sender: [{}], chan: [{}], msg: [{}]", sender.as_str(), chan.as_str(), msg.as_str());
                },
                _ => eprintln!("unreachable expression, this is a bug!")
            },
            Err(e) => {
                eprintln!("err: <Console Input> {}", e);
            }
        }
    }
}
