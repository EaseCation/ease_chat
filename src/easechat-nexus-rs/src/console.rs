use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "console_rule.pest"]
struct NexusParser;
