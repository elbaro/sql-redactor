use sql_redactor::*;

fn main() {
    let t = std::time::Instant::now();
    const N: usize = 10000;
    for _ in 0..N {
        // redact(&MySqlDialect {}, "SELECT * FROM foo WHERE bar = 1").unwrap();
        redact(
            &MySqlDialect {},
            "SELECT * FROM users 
        WHERE age > 18 
        AND city = 'New York' 
        ORDER BY last_name ASC;",
        )
        .unwrap();
    }
    dbg!(t.elapsed().div_f64(N as f64));
}
