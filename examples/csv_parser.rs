use std::fs;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "examples/csv.pest"]
struct CSVParser;

fn main() {
    let unparsed_file = fs::read_to_string("examples/numbers.csv").expect("cannot read file");

    let file = CSVParser::parse(Rule::file, &unparsed_file)
        .expect("cannot parse file")
        .next()
        .unwrap();

    let mut field_sum: f64 = 0.;
    let mut record_count: u64 = 0;

    for record in file.into_inner() {
        match record.as_rule() {
            Rule::record => {
                record_count += 1;

                for field in record.into_inner() {
                    field_sum += field.as_str().parse::<f64>().unwrap();
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("Number of record: {}", record_count);
    println!("Sum of fields: {}", field_sum);

    // println!("{:#?}", file);
}
