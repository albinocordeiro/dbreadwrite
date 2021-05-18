table! {
    accounts (id) {
        id -> Int4,
        balance -> Int8,
    }
}

table! {
    burns (id) {
        id -> Int4,
        time -> Timestamp,
        amount -> Int8,
        account_id -> Nullable<Int4>,
    }
}

table! {
    mints (id) {
        id -> Int4,
        time -> Timestamp,
        amount -> Int8,
        account_id -> Nullable<Int4>,
    }
}

table! {
    transfers (id) {
        id -> Int4,
        time -> Timestamp,
        amount -> Int8,
        from_account_id -> Nullable<Int4>,
        to_account_id -> Nullable<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts,
    burns,
    mints,
    transfers,
);
