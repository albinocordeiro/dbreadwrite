CREATE TABLE burns (
    id INT PRIMARY KEY,
    time TIMESTAMP NOT NULL,
    amount BIGINT NOT NULL,
    account_id INT
)