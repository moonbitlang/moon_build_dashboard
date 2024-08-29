CREATE TABLE statistics (
    repo            TEXT NOT NULL,
    rev             TEXT NOT NULL,
    command         TEXT NOT NULL,
    moon_version    TEXT NOT NULL,
    moonc_version   TEXT NOT NULL,
    status          INTEGER NOT NULL,
    elapsed         INTEGER,
    start_time      TIMESTAMP_MS,
    run_id          TEXT NOT NULL,
    run_number      TEXT NOT NULL
);

INSERT INTO statistics
SELECT *
FROM read_json('data.jsonl', format = 'unstructured', timestampformat = 'yyyy-MM-dd HH:mm:ss.SSS');

select repo, status, elapsed, start_time from statistics;


CREATE TEMPORARY TABLE latest_grouped_status AS
SELECT 
    CAST(start_time AS DATE) AS dt,
    start_time,
    repo,
    command,
    moonc_version,
    run_id,
    CASE 
        WHEN status = 0 THEN 'ok'
        ELSE 'failed'
    END AS status
FROM (
    SELECT 
        repo,
        command,
        moonc_version,
        start_time,
        status,
        run_id,
        ROW_NUMBER() OVER (
            PARTITION BY repo, command, moonc_version 
            ORDER BY start_time DESC
        ) AS row_num
    FROM
        statistics
    WHERE
        command IN ('Check', 'Build', 'Test', 'Bundle')
) AS repo_status
WHERE row_num = 1;

SELECT * 
FROM latest_grouped_status 
ORDER BY repo, command;
