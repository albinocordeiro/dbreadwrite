CREATE TABLE transfers (
    id INT PRIMARY KEY,
    time TIMESTAMP NOT NULL,
    amount BIGINT NOT NULL,
    from_account_id INT,
    to_account_id INT
)