use std::collections::HashMap;
use std::error::Error;

use csv;

pub(crate) type Record = HashMap<String, String>;

pub(crate) fn read_csv(csv_file: &[u8]) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut records: Vec<Record> = Vec::<Record>::new();
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(csv_file);
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record)
    }
    Ok(records)
}
