use crate::pgconn::establish_connection;
use color_eyre::eyre::{eyre, Result};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use log::{info, trace};
use notify::{watcher, DebouncedEvent, INotifyWatcher, RecursiveMode, Watcher};
use serde_json::{from_str, Map, Value};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

pub struct SchemaManager {
    event_types: Map<String, Value>,
    event_types_file: String,
    file_watcher: INotifyWatcher,
    file_watcher_recv: Receiver<DebouncedEvent>,
}

impl SchemaManager {
    pub fn new(file_name: &str) -> Result<Self> {
        let (tx, rx) = channel();
        let mut newobj = Self {
            event_types_file: file_name.to_owned(),
            event_types: Map::new(),
            file_watcher: watcher(tx, Duration::from_secs(5))?,
            file_watcher_recv: rx,
        };

        // Create a watcher for change events on the event type file on a separate thread
        newobj
            .file_watcher
            .watch(&file_name, RecursiveMode::NonRecursive)?;

        // Initial file load and schema update attempt
        newobj.event_types = load_event_types_from_file(&file_name)?;
        newobj.update_schema()?;
        Ok(newobj)
    }

    pub fn check_update_schema(&mut self) -> Result<()> {
        //! Non-blocking check if json file has been recently written to. If it's the case it'll update the db schema
        match self.file_watcher_recv.try_recv() {
            Ok(path_watch_event) => {
                match path_watch_event {
                    DebouncedEvent::Write(_) => {
                        info!("Detected change in json file");
                        self.update_schema()?;
                    }
                    _ => {}
                };
            }
            Err(e) => match e {
                TryRecvError::Disconnected => {
                    return Err(eyre!("The file watcher is broken"));
                }
                TryRecvError::Empty => {} // No recent file changes detected, continue ...
            },
        };

        Ok(())
    }

    fn update_schema(&mut self) -> Result<()> {
        //! 1. Load event types from file
        //! 2. For each event type build SQL transaction to create table if it doesn't exist yet
        //! 3. Update self event_types

        let updated_event_types = load_event_types_from_file(&self.event_types_file)?;
        for event_type in &updated_event_types {
            let conn: PgConnection = establish_connection();
            
            // Add table if missing
            conn.build_transaction()
                .serializable()
                .run::<_, diesel::result::Error, _>(|| {
                    let prepared_query_str = query_from_event_type(&event_type).unwrap();
                    sql_query(prepared_query_str).execute(&conn)?;
                    // Create Index if it doesn't exist
                    
                    Ok(())
                })?;

            // Index can't be created inside a transaction block
            sql_query(format!("CREATE INDEX CONCURRENTLY IF NOT EXISTS {}_tsidx ON {}(time)", &event_type.0[..], &event_type.0[..], )).execute(&conn)?;
            
            // Add column if missing
            conn.build_transaction()
                .serializable()
                .run::<_, diesel::result::Error, _>(|| {
                    if let Value::Object(map) = event_type.1 {
                        if let Value::Object(type_mapping) = &map["type_mapping"] {
                            for colval in type_mapping {
                                if let Value::String(data_type) = colval.1 {
                                    let prepared_query_str = query_from_column_data_type(
                                        &event_type.0.to_owned(),
                                        &colval.0.to_owned(),
                                        &data_type,
                                    )
                                    .unwrap();
                                    sql_query(prepared_query_str).execute(&conn)?;
                                }
                            }
                        }
                    }
                    Ok(())
                })?;
        }
        self.event_types = updated_event_types.clone();
        match self.is_event_types_valid() {
            true => Ok(()),
            false => Err(eyre!(
                "The database schema does not seem to be valid or ready"
            )),
        }
    }

    pub fn get_event_types(&self) -> &Map<String, Value> {
        &self.event_types
    }

    fn is_event_types_valid(&self) -> bool {
        !self.event_types.is_empty()
    }
}

pub fn load_event_types_from_file(event_types_file: &str) -> Result<Map<String, Value>> {
    let mut file = File::open(event_types_file)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let obj: Map<String, Value> = from_str(&file_content)?;
    trace!("{:?}", obj);
    if obj.is_empty() {
        return Err(eyre!(
            "Could not parse a single event_type from {}",
            event_types_file
        ));
    }

    Ok(obj)
}

fn query_from_event_type(jsonobj: &(&std::string::String, &serde_json::Value)) -> Result<String> {
    let table_name = String::from(jsonobj.0);
    let mut columns = String::new();
    if let Value::Object(map) = jsonobj.1 {
        if let Value::Object(type_mapping) = &map["type_mapping"] {
            let mut counter: usize = 0;
            for colval in type_mapping {
                let col_name = String::from(colval.0);
                if let Value::String(data_type) = colval.1 {
                    columns.push_str(&format!("{} {}", col_name, data_type));
                    if counter != type_mapping.len() - 1 {
                        columns.push_str(",");
                    }
                }
                counter += 1;
            }
        }
    }
    let res = format!("CREATE TABLE IF NOT EXISTS {} ({})", &table_name, &columns);
    trace!("{}", res);
    Ok(res)
}

fn query_from_column_data_type(
    table_name: &str,
    column_name: &str,
    data_type: &str,
) -> Result<String> {
    let res = format!(
        "ALTER TABLE {} ADD COLUMN IF NOT EXISTS {} {}",
        table_name, column_name, data_type
    );
    trace!("{}", res);
    Ok(res)
}

#[test]
fn test_query_from_column_data_type() -> Result<()> {
    let q = query_from_column_data_type("mint_coins", "time", "timestamp")?;
    assert_eq!(
        &q,
        "ALTER TABLE mint_coins ADD COLUMN IF NOT EXISTS time timestamp"
    );
    Ok(())
}
#[test]
fn test_load_event_types_from_file() -> Result<()> {
    let map = load_event_types_from_file("./typemapping/type_mapping.json")?;
    assert!(map.len() == 3usize);
    Ok(())
}

#[test]
fn test_query_from_event_type() -> Result<()> {
    let map = load_event_types_from_file("./typemapping/type_mapping.json")?;
    let query = query_from_event_type(&(&"mint_coins".to_owned(), &map["mint_coins"]))?;
    println!("{}", query);
    assert_eq!(
        &query,
        "CREATE TABLE IF NOT EXISTS mint_coins (account_id int,amount bigint,time timestamp)"
    );
    Ok(())
}
