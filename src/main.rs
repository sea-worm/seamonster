use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::{env, io};

use ::csv::{ReaderBuilder, Trim, Writer};

use crate::bank::Engine;
use crate::csv::{CSVSummaryRecord, CSVTransaction};
use crate::transaction::{ClientId, MoneyType, Transaction, TransactionData, TransactionId};
use crate::Transaction::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

mod bank;
mod csv;
mod transaction;

fn main() {
    let input_csv = env::args().nth(1).expect("No CSV input file given");
    let csv_file = File::open(input_csv).expect("Error opening the CSV file");

    let csv_file_buffer = BufReader::new(csv_file);
    let stdout = io::stdout();
    if let Err(err) = process_csv(csv_file_buffer, stdout) {
        println!("ERROR processing CSV: {}", err);
        exit(1);
    }
}

fn process_csv<R: io::Read, W: io::Write>(csv_buffer: R, output: W) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(csv_buffer);
    let mut engine = Engine::create();
    for record in rdr.deserialize() {
        // keep things simple here by aborting with hard errors when parsing the CSV rows fails
        let csv_transaction: CSVTransaction = record?;
        let tx = csv_transaction.to_transaction()?;
        // However, softly skip transactions if they couldn't be processed (e.g. NotEnoughBalance)
        engine.update(tx).unwrap_or_else(|e| {
            eprintln!("Error processing {:?}: {:?}", csv_transaction, e);
        })
    }
    let mut wtr = Writer::from_writer(output);
    for (client_id, client_summary) in engine.iter() {
        let held = client_summary.held;
        let available = client_summary.available;
        let total = available + held;
        wtr.serialize(CSVSummaryRecord {
            client: client_id.0,
            locked: client_summary.locked,
            held: held.into(),
            available: available.into(),
            total: total.into(),
        })?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::process_csv;
    use std::error::Error;
    use std::io::BufReader;

    #[ignore]
    #[test]
    fn test_csv() {
        let csv_file = process_csv_str("TEST_FILE_REMOVED").unwrap();
        assert_eq!(csv_file, "REMOVED");
    }

    fn process_csv_str(input: &str) -> Result<String, Box<dyn Error>> {
        let test_csv = BufReader::new(input.as_bytes());
        let mut buf = Vec::new();
        process_csv(test_csv, &mut buf)?;
        let csv_file = String::from_utf8(buf)?;
        // lines are not sorted -> split and do a simple sort
        let (header, rows) = csv_file.trim().split_once("\n").ok_or("No CSV header")?;
        let mut rows = rows.split("\n").collect::<Vec<_>>();
        rows.sort();
        let rows = rows.join("\n");
        Ok(format!("{header}\n{rows}"))
    }
}
