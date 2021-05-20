//! Generate random writes and random reads
use crate::pgconn::establish_connection;
use color_eyre::eyre::{Result, eyre};
use diesel::pg::data_types::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::*;
use log::trace;
use rand::prelude::*;
use serde_json::{Map, Value};
use chrono::prelude::*;
use chrono::Duration;

/// Generate a random write
pub fn commit_random_event( event_types: &Map<String, Value>) -> Result<()> {
    let mut rng = thread_rng();

    let event_type_names: Vec<&String> = event_types.keys().collect();

    let random_idx = rng.gen_range(0..event_types.len());
    let event_name = event_type_names[random_idx];
    let event_type: &Value = &event_types[event_name];

    trace!("Random pick event type: {:?}", &event_name);

    // Generate random values for cols data types

    if let Value::Object(map) = event_type {
        if let Value::Object(type_mapping) = &map["type_mapping"] {
            let insert_sql = sql_from_type_mapping(event_name, &type_mapping)?;
            trace!("{}", insert_sql);
            let conn = establish_connection();
            conn.build_transaction()
                .repeatable_read()
                .run::<_, diesel::result::Error, _>(|| {
                    sql_query(insert_sql).execute(&conn)?;
                    Ok(())
                })?; 
            return Ok(());
        }
    }

    Err(eyre!("Invalid event_type object {:?}", event_type))
}


fn sql_from_type_mapping(table_name: &str, type_mapping: &Map<String, Value>) -> Result<String> {
    let mut rng = thread_rng();
    let mut columns_list = String::new();
    let mut values = String::new();
    let mut counter: usize = 0;
    for col_definition in type_mapping {
        columns_list.push_str(col_definition.0);
        if let Value::String(data_type) = col_definition.1 {
            let random_value: String = match &data_type[..] {
                "timestamp" => {
                    (Utc::now() - Duration::milliseconds(rng.gen_range(1i64..1000000))).format("'%Y-%m-%d %H:%M:%S'").to_string()
                },
                "bigint" => {
                    rng.gen_range(1i64..1000000000).to_string()
                },
                "int" => {
                    rng.gen_range(1i32..1000).to_string()
                }
                _ => String::from("")
            };
            values.push_str(&random_value);
        }
        if counter != type_mapping.len() - 1usize {
            columns_list.push_str(", ");
            values.push_str(", ");
        }
        counter += 1;
    }

    let res = format!("INSERT INTO {}({}) VALUES ({})", &table_name, &columns_list, &values);
    Ok(res)
}