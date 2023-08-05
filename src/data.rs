use std::collections::HashMap;
use std::error::Error;
use csv;

pub type Record = HashMap<String, String>;

pub fn read_csv(file_path : String) -> Result<Vec<Record>, Box<dyn Error>>{
    let mut records : Vec<Record> = Vec::<Record>::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(file_path)?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record)
    }
    Ok(records)
}