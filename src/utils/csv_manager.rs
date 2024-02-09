use std::{error::Error, fs::File};

use csv::{Writer, ReaderBuilder, Trim, WriterBuilder};

use crate::models::processed_trade::ProcessedTrade;

pub fn write_csv(data: ProcessedTrade, file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_path)?;
    // Write the ProcessedTrade data to the CSV
    wtr.serialize(data)?;
    wtr.flush()?;
    Ok(())
}

pub fn update_csv(data: ProcessedTrade, file_path: &str) -> Result<(), Box<dyn Error>> {
    // Open the CSV file for reading
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(file);

    // Read all records from the CSV into memory
    let mut records: Vec<ProcessedTrade> = Vec::new();
    for result in rdr.deserialize() {
        let record: ProcessedTrade = result?;
        records.push(record);
    }

    // Find and update the relevant record
    for record in &mut records {
        if record.pair_address == data.pair_address {
            // Update the record's fields from the data
            *record = data.clone();
            break;  // Assuming only one record will match
        }
    }

    // Open the CSV file for writing (this will overwrite the existing file)
    let mut wtr = WriterBuilder::new().from_path(file_path)?;

    // Write the updated records back to the CSV
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;

    Ok(())
}