# dbreadwrite
## Problem Statement
There are 2 parts to this test. In part 1, you’ll create a database
writer application and in part 2, you’ll create a database reader application. These should be
separate applications and are expected to be able to run simultaneously with multiple instances
of each.

### Writer
This application’s role is to process events and store them in an SQL database. It should insert
1 new record into the database every 5 seconds with random values. (This is to simulate events
coming in and being processed)
The structure of the various events is defined in the type_mapping.json file provided at the end
of this document. It contains the type mappings for 3 events: ‘mint_coins’, ‘transfer_coins’,
‘burn_coins’. The application should be able to handle additions of events and/or fields in this
file.
The application must read the json and update the database schema accordingly if applicable.
Existing data previously created with a different type mappings should be preserved.
Any identical events that get processed should produce a single record in the database (no
duplicate rows)

### Reader
This application must query the database 10 times per second. Each query should be for a
random event type and filter a random time frame between a start and end time. (This is to
simulate high traffic of users sending requests)

## Analysis

### Database design
From the problem statement I conclude that the the data base would have to start with at least one table, the accounts table, to keep records of accounts and their balances. Also, based on the events definition json file, a table per event type should be added.  

### Writer
The writer needs to account for the fact that ther could be multiple instances of the writer and reader. That means that concurrent writes/updates could cause inconsistency in the data, in particular, in the accounts table where balances could be corrupted.

Another main topic in the writer is that it needs to keep track of changes in the events definition json file and update the database schema with new fields and event types.

### Reader
A single instance of the reader generates 10 queries per second, thus, performance is key. Indexes on the time stamp column should be very helpful in this situation.

[![Crates.io](https://img.shields.io/crates/v/dbreadwrite.svg)](https://crates.io/crates/dbreadwrite)
[![Docs.rs](https://docs.rs/dbreadwrite/badge.svg)](https://docs.rs/dbreadwrite)
[![CI](https://github.com/albinocordeiro/dbreadwrite/workflows/Continuous%20Integration/badge.svg)](https://github.com/albinocordeiro/dbreadwrite/actions)
[![Coverage Status](https://coveralls.io/repos/github/albinocordeiro/dbreadwrite/badge.svg?branch=main)](https://coveralls.io/github/albinocordeiro/dbreadwrite?branch=main)

## Installation
```bash 

echo DATABASE_URL=postgres://username:password@localhost/dbreadwrite > .env
# Add the excutables folder to the path
export PATH=/home/user_name/projects/dbreadwrite/target/debug:$PATH 
```
### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo build`
* run `cargo doc --open`


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
