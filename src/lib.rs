pub use sqlparser::dialect::*;

use std::fmt::Write;
use std::ops::ControlFlow;

use sqlparser::{ast::visit_expressions_mut, parser::Parser};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse the query")]
    FailedToParse(#[from] sqlparser::parser::ParserError),
    #[error("failed to stringify the redacted query; check the memory usage")]
    FailedToStringify(#[from] std::fmt::Error),
}

/// Replace all Value nodes in an AST with '?'
pub fn redact(dialect: &dyn Dialect, sql: &str) -> Result<String, Error> {
    let statements = Parser::parse_sql(dialect, sql)?;
    let mut redacted = String::with_capacity(sql.len());
    for mut stmt in statements {
        visit_expressions_mut(&mut stmt, |expr| {
            if let sqlparser::ast::Expr::Value(value) = expr {
                *value = sqlparser::ast::Value::Placeholder(String::from("?"));
            }
            ControlFlow::<()>::Continue(())
        });
        write!(redacted, "{};", stmt)?;
    }

    Ok(redacted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql() {
        let redacted = redact(&MySqlDialect {}, "SELECT * FROM foo WHERE bar = 1").unwrap();
        assert_eq!(redacted, "SELECT * FROM foo WHERE bar = ?;");

        let redacted = redact(
            &MySqlDialect {},
            "SELECT user, article, 5 FROM articles WHERE user = 100 AND is_deleted = false",
        )
        .unwrap();
        assert_eq!(
            redacted,
            "SELECT user, article, ? FROM articles WHERE user = ? AND is_deleted = ?;"
        );

        let redacted = redact(
            &MySqlDialect {},
            "SELECT * FROM users 
            WHERE age > 18 
            AND city = 'New York' 
            ORDER BY last_name ASC;",
        )
        .unwrap();
        assert_eq!(
            redacted,
            "SELECT * FROM users WHERE age > ? AND city = ? ORDER BY last_name ASC;"
        );

        let redacted = redact(
            &MySqlDialect {},
            "UPDATE customers 
            SET email = 'newemail@example.com', last_purchase_date = '2022-03-31' 
            WHERE customer_id = 12345;",
        )
        .unwrap();
        assert_eq!(
            redacted,
            "UPDATE customers SET email = ?, last_purchase_date = ? WHERE customer_id = ?;"
        );

        let redacted = redact(
            &MySqlDialect {},
            "INSERT INTO users (name, email, age) 
            VALUES ('John Doe', 'johndoe@example.com', 25);",
        )
        .unwrap();
        assert_eq!(
            redacted,
            "INSERT INTO users (name, email, age) VALUES (?, ?, ?);"
        );

        let redacted = redact(
            &MySqlDialect {},
            "DELETE FROM users 
            WHERE email = 'johndoe@example.com';",
        )
        .unwrap();
        assert_eq!(redacted, "DELETE FROM users WHERE email = ?;");

        let redacted = redact(
            &MySqlDialect {},
            "SELECT c.name AS category, AVG(p.price) AS avg_price 
            FROM products p 
            JOIN categories c ON p.category_id = c.id 
            GROUP BY c.name;",
        )
        .unwrap();
        assert_eq!(redacted, "SELECT c.name AS category, AVG(p.price) AS avg_price FROM products AS p JOIN categories AS c ON p.category_id = c.id GROUP BY c.name;");

        let redacted = redact(
            &MySqlDialect {},
            "SELECT name, email, age 
            FROM users 
            WHERE age BETWEEN 18 AND 30 
            ORDER BY RAND() LIMIT 5;",
        )
        .unwrap();
        assert_eq!(
            redacted,
            "SELECT name, email, age FROM users WHERE age BETWEEN ? AND ? ORDER BY RAND() LIMIT ?;"
        );

        let redacted = redact(
            &MySqlDialect {},
            "SELECT c.name AS category, AVG(p.price) AS avg_price 
            FROM products p 
            JOIN categories c ON p.category_id = c.id 
            GROUP BY c.name;
            SELECT name, email, age 
            FROM users 
            WHERE age BETWEEN 18 AND 30 
            ORDER BY RAND() LIMIT 5;",
        )
        .unwrap();
        assert_eq!(
            redacted,
            "SELECT c.name AS category, AVG(p.price) AS avg_price FROM products AS p JOIN categories AS c ON p.category_id = c.id GROUP BY c.name;SELECT name, email, age FROM users WHERE age BETWEEN ? AND ? ORDER BY RAND() LIMIT ?;"
        );

        assert!(matches!(
            redact(&MySqlDialect {}, "this is not a sql."),
            Err(Error::FailedToParse(_))
        ));
    }
}
