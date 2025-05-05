use std::path::Path;
use std::fs::File;
use eyre::Result;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use arrow::datatypes::FieldRef;
use serde_arrow::schema::{SchemaLike, TracingOptions};
use crate::{PriceFetcherResult, PricesMetadata};


pub fn write_prices_to_parquet(records: &Vec<PriceFetcherResult>, out_path: &Path) -> Result<()> {
    let fields = Vec::<FieldRef>::from_samples(records, TracingOptions::default())?;
    let batch = serde_arrow::to_record_batch(&fields, &records)?;

    let file = File::create(out_path)?;
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
    
    writer.write(&batch)?;
    writer.close()?;
    
    Ok(())
}

pub fn write_prices_metadata(metadata: PricesMetadata, out_path: &Path) -> Result<()> {
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(out_path, metadata_json)?;
    Ok(())
}
