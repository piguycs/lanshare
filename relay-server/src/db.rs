#[must_use]
pub fn gen_mem_db() -> sqlite::Connection {
    sqlite::open(":memory:").unwrap()
}
