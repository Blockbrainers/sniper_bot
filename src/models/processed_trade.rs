use csv::{Writer, ReaderBuilder, Trim, WriterBuilder};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions, self};
use std::option::Option;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils;

const FILE_PATH: &str = "data.csv";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedTrade {
    // Base info
    pub network_chain_id: String,
    pub network_name: String,
    pub exchange_name: String,
    pub pair_address: Address,
    pub token_address: Address,
    pub base_token_address: Address,
    pub last_update: String,
    pub status: TradeStatus,

    // Need update methods
    pub substatus: Option<TradeSubStatus>,
    pub security_score: Option<f64>,
    pub amount_bought: Option<f64>,
    pub amount_sold: Option<f64>,
    pub exchange_fee_paid: Option<f64>,
    pub gas_fee_paid: Option<f64>,
    pub profit_or_loss: Option<f64>,
    pub multiplier: Option<f64>,
    pub bot_wallet_balance: Option<f64>,
}

impl ProcessedTrade {
    // Constructor for new trade
    pub fn new(
        network_chain_id: String,
        network_name: String,
        exchange_name: String,
        pair_address: Address,
        token_address: Address,
        base_token_address: Address,
    ) -> Result<Self, Box<dyn Error>> {
        let trade = ProcessedTrade {
            network_chain_id,
            network_name,
            exchange_name,
            pair_address,
            token_address,
            base_token_address,
            last_update: formatted_time(),
            status: TradeStatus::OpenPosition, // defaulting to OpenPosition, can be changed as needed
            // Initialize all other fields as None or default
            substatus: None,
            security_score: None,
            amount_bought: None,
            amount_sold: None,
            exchange_fee_paid: None,
            gas_fee_paid: None,
            profit_or_loss: None,
            multiplier: None,
            bot_wallet_balance: None,
        };

        // Save the new trade to CSV
        trade.append_to_csv()?;

        Ok(trade)
    }

    // Update to OpenPosition status
    pub fn open_position(&mut self, amount: f64, security_score: f64) -> Result<(), Box<dyn Error>> {
        self.status = TradeStatus::OpenPosition;
        self.amount_bought = Some(amount);
        self.security_score = Some(security_score);
        self.last_update = formatted_time();
        self.update_csv()
    }

    // Update to ClosedPosition status
    pub fn closed_position(&mut self, amount_sold: f64, profit_or_loss: f64) -> Result<(), Box<dyn Error>> {
        self.status = TradeStatus::ClosedPosition;
        self.amount_sold = Some(amount_sold);
        self.profit_or_loss = Some(profit_or_loss);
        self.last_update = formatted_time();
        self.update_csv()
    }

    // Update to Canceled status
    pub fn canceled(&mut self, substatus: TradeSubStatus) -> Result<(), Box<dyn Error>> {
        self.status = TradeStatus::Canceled;
        self.substatus = Some(substatus);
        self.last_update = formatted_time();
        self.update_csv()
    }

    // Internal method to update the CSV with the current state of the trade
    fn update_csv(&self) -> Result<(), Box<dyn Error>> {
            // Open the CSV file for reading
        let file = File::open(FILE_PATH)?;
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
            if record.pair_address == self.pair_address {
                // Update the record's fields from the data
                *record = self.clone();
                break;  // Assuming only one record will match
            }
        }

        // Open the CSV file for writing (this will overwrite the existing file)
        let mut wtr = WriterBuilder::new().from_path(FILE_PATH)?;

        // Write the updated records back to the CSV
        for record in records {
            wtr.serialize(record)?;
        }
        wtr.flush()?;

        Ok(())
    }

    // Internal method to write the trade data to the CSV when created
    fn write_csv(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(FILE_PATH)?;
        // Write the ProcessedTrade data to the CSV
        wtr.serialize(self)?;
        wtr.flush()?;

        Ok(())
    }

    fn append_to_csv(&self) -> Result<(), Box<dyn Error>> {
        // Check if the file exists
        let file_exists = Path::new(FILE_PATH).exists();
    
        // Open the CSV file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(FILE_PATH)?;
    
        let mut wtr = WriterBuilder::new().from_writer(file);
    
        // If the file doesn't exist, write the header
        if !file_exists {
            wtr.write_record(&[
                "network_chain_id",
                "network_name",
                "exchange_name",
                "pair_address",
                "token_address",
                "base_token_address",
                "last_update",
                "status",
                "substatus",
                "security_score",
                "amount_bought",
                "amount_sold",
                "exchange_fee_paid",
                "gas_fee_paid",
                "profit_or_loss",
                "multiplier",
                "bot_wallet_balance",
            ])?;
        }
    
        // Write the ProcessedTrade data to the CSV
        wtr.serialize(self)?;
        wtr.flush()?;
    
        Ok(())
    }
    
}

fn is_file_empty(file_path: &str) -> Result<bool, Box<dyn Error>> {
    // Check if the file exists
    if !Path::new(file_path).exists() {
        return Ok(true);
    }

    // Check if the file is empty (has no content)
    let metadata = fs::metadata(file_path)?;
    let is_empty = metadata.len() == 0;

    Ok(is_empty)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeStatus {
    OpenPosition,
    ClosedPosition,
    Canceled,
    // Add more as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSubStatus {
    NotBaseTokenPair,
    ExistingPoolsFound,
    FailedSecurityCheck,
    InsufficientFunds,
    // Add more as needed
}

fn formatted_time() -> String {
    let now = SystemTime::now();

    // Convert SystemTime to a String in a specific format
    let formatted_time = match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().to_string(), // Convert to seconds and then to String
        Err(_) => String::from("Invalid time"), // Handle error if SystemTime is earlier than UNIX_EPOCH
    };
    formatted_time
}