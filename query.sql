CREATE TABLE statistics (
    repo            TEXT NOT NULL,
    rev             TEXT NOT NULL,
    command         TEXT NOT NULL,
    moon_version    TEXT NOT NULL,
    moonc_version   TEXT NOT NULL,
    elapsed         INTEGER,
    start_time      TIMESTAMP_MS,
    run_id          TEXT NOT NULL,
    run_number      TEXT NOT NULL
);

INSERT INTO statistics
SELECT *
FROM read_json('data.jsonl', format = 'unstructured');

select * from statistics;
