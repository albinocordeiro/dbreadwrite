//! Generate random writes and random reads
use crate::pgconn::establish_connection;
use color_eyre::eyre::{Result, eyre};
use diesel::prelude::*;
use diesel::sql_query;
use log::trace;
use rand::prelude::*;
use serde_json::{Map, Value};
use chrono::prelude::*;
use chrono::Duration;


/// Generate a random read
pub fn execute_random_read_query(event_types: &Map<String, Value>) -> Result<()> {
    let mut rng = thread_rng();
    let (event_name, _): (&str, &Value) = pick_random_event_type(event_types)?;

    // SELECT * FROM event_name WHERE time BETWEEN now-random AND now 
    let conn = establish_connection();
    let now = Utc::now().format("'%Y-%m-%d %H:%M:%S'").to_string();
    let before = (Utc::now() - Duration::milliseconds(rng.gen_range(1i64..1000000))).format("'%Y-%m-%d %H:%M:%S'").to_string();
    let select_sql = format!("SELECT time,amount FROM {} WHERE time BETWEEN {} AND {}", &event_name, &before, &now);
    sql_query(&select_sql).execute(&conn)?;
    Ok(())
}

/// Generate a random write
pub fn commit_random_event(event_types: &Map<String, Value>) -> Result<()> {
    let (event_name, event_type): (&str, &Value) = pick_random_event_type(event_types)?;

    // Generate random values for the columns' data types
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

fn pick_random_event_type(event_types: &Map<String, Value>) -> Result<(&str, &Value)> {
    let mut rng = thread_rng();

    let event_type_names: Vec<&String> = event_types.keys().collect();

    let random_idx = rng.gen_range(0..event_types.len());
    let event_name = event_type_names[random_idx];
    let event_type: &Value = &event_types[event_name];

    trace!("Random picked event type: {}", &event_name);
    
    Ok((&event_name[..], event_type))
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