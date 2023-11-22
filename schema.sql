create table similar_transactions
(
    "hash"          text not null constraint similar_transactions_pk primary key,
    "chain"         text,
    "aaType"        text,
    "status"        text,
    "timestamp"     timestamp
);