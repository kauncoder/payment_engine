use std::{collections::HashMap, env, error::Error, fs::File, io};
mod payments;
use csv::{ReaderBuilder, Trim, Writer};
use payments::{ClientMap, Transaction, TxnAmountMap};
use std::fs::metadata;

const FILE_SIZE_LIMIT: u64 = 500000000;

fn read_csv(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut client_data: ClientMap = HashMap::new(); //modify this everytime there is a change in the alu
    let mut txn_map: TxnAmountMap = HashMap::new();
    let metadata = metadata(file_path).unwrap();
    let file_size = metadata.len();
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .from_reader(file);

    //set file size limit to 500mb for large csv files
    if file_size > FILE_SIZE_LIMIT {
        for result in rdr.deserialize() {
            let txn: Transaction = result?;
            //perform operation on the txn
            txn.process(&mut client_data, &mut txn_map)?;
        }
    } else {
        let txn_list: Vec<Transaction> = rdr.deserialize().collect::<Result<_, _>>().unwrap();
        //perform operations on the list of txns
        for txn in txn_list {
            txn.process(&mut client_data, &mut txn_map)?;
        }
    }
    write_csv(client_data)?;
    Ok(())
}

fn write_csv(client_data: ClientMap) -> Result<(), Box<dyn Error>> {
    // Create a CSV writer that writes to stdout
    let mut wtr = Writer::from_writer(io::stdout());

    // Write each person struct as a record in CSV format
    for (_, client) in client_data {
        wtr.serialize(client)?;
    }

    // Flush the writer to ensure everything is written out
    wtr.flush()?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).unwrap();
    let _ = read_csv(&filename).unwrap();
}
