# sql-redactor

Normalize and redact SQL queries.
Supports many [dialects](https://docs.rs/sqlparser/latest/sqlparser/dialect/index.html) such as mysql, postgres, clickhouse, or hive.

This is useful for adding the observability to your database.

It is hard to find the higest QPS query when the queries are parameterized:
- `SELECT * FROM users where user_id = 1000` with QPS 5
- `SELECT * FROM users where user_id = 1001` with QPS 3
- `SELECT * FROM users where user_id = 1002` with QPS 8
- ..
- `SELECT * FROM articles where article_id = 2000` with QPS 2
- `SELECT * FROM articles where article_id = 2001` with QPS 50
- `SELECT * FROM articles where article_id = 2002` with QPS 3

The parameters can be obscured to provide a better insight: 
- `SELECT * FROM users where user_id = ?` with QPS 3,000
- `SELECT * FROM articles where article_id = ?` with QPS 2,000

## Usage

```sh
cargo add sql-redactor
```

```rs
use sql_redactor::redact;
use sql_redactor::dialect::MySqlDialect;

let sql = "SELECT * FROM users 
            WHERE age > 18 
            AND city = 'New York' 
            ORDER BY last_name ASC;";

let redacted = "SELECT * FROM users WHERE age > ? AND city = ? ORDER BY last_name ASC;";

assert_eq!(redact(&MySqlDialect {}, sql).unwrap(), redacted);
```

## Bench

The redaction is fast compared to typical db latencies.


AMD Ryzen 9 3900X

30us
```
SELECT * FROM foo WHERE bar = 1
```

60~70 us
```
SELECT * FROM users 
        WHERE age > 18 
        AND city = 'New York' 
        ORDER BY last_name ASC;
```
