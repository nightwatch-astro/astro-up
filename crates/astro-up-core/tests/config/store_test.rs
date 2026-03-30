use astro_up_core::config::ConfigStore;
use rusqlite::Connection;

#[test]
fn store_creates_table_on_new() {
    let conn = Connection::open_in_memory().unwrap();
    let _store = ConfigStore::new(conn).unwrap();
    // If we get here, table was created
}

#[test]
fn store_set_and_get() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    store.set("network.timeout", "60s").unwrap();
    let val = store.get("network.timeout").unwrap();
    assert_eq!(val, Some("60s".to_string()));
}

#[test]
fn store_get_missing_returns_none() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    let val = store.get("nonexistent").unwrap();
    assert_eq!(val, None);
}

#[test]
fn store_set_overwrites() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    store.set("key", "v1").unwrap();
    store.set("key", "v2").unwrap();
    assert_eq!(store.get("key").unwrap(), Some("v2".to_string()));
}

#[test]
fn store_list_returns_all_sorted() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    store.set("b.key", "2").unwrap();
    store.set("a.key", "1").unwrap();

    let list = store.list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0], ("a.key".to_string(), "1".to_string()));
    assert_eq!(list[1], ("b.key".to_string(), "2".to_string()));
}

#[test]
fn store_reset_removes_key() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    store.set("key", "value").unwrap();
    store.reset("key").unwrap();
    assert_eq!(store.get("key").unwrap(), None);
}

#[test]
fn store_reset_nonexistent_is_noop() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    // Should not error
    store.reset("never.set").unwrap();
}

#[test]
fn store_reset_all() {
    let conn = Connection::open_in_memory().unwrap();
    let store = ConfigStore::new(conn).unwrap();

    store.set("a", "1").unwrap();
    store.set("b", "2").unwrap();
    store.reset_all().unwrap();
    assert!(store.list().unwrap().is_empty());
}
