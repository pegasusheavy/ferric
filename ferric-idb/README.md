# Ferric IDB

An ergonomic async wrapper for IndexedDB in WebAssembly.

## Why?

IndexedDB has a notoriously difficult callback-based API. This crate provides:

- **Async/await API** - No callbacks, just clean async Rust code
- **Type-safe** - Serde integration for automatic serialization/deserialization
- **Ergonomic** - Builder patterns for database and store configuration
- **Complete** - Full IndexedDB feature support (indexes, cursors, transactions)

## Quick Start

```rust
use ferric_idb::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn example() -> IdbResult<()> {
    // Open/create a database
    let db = Database::open("my-app")
        .version(1)
        .on_upgrade(|ctx| {
            // Create object stores during upgrade
            ctx.create_object_store("users")
                .key_path("id")
                .auto_increment(true)
                .index("email", "email")
                .unique_index("name", "name")
                .build()?;
            Ok(())
        })
        .build()
        .await?;

    // Insert a record
    let tx = db.transaction_on("users").rw().build()?;
    let store = tx.store("users")?;

    store.put(&User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    }).await?;

    tx.commit().await?;

    // Query a record
    let tx = db.transaction_on("users").build()?;
    let store = tx.store("users")?;

    let user: Option<User> = store.get(1).await?;
    println!("User: {:?}", user);

    Ok(())
}
```

## Features

### Database Management

```rust
// Open with version and upgrade handler
let db = Database::open("mydb")
    .version(2)
    .on_upgrade(|ctx| {
        // Runs when version changes
        if ctx.old_version < 1 {
            ctx.create_object_store("users")
                .key_path("id")
                .build()?;
        }
        if ctx.old_version < 2 {
            ctx.create_object_store("posts")
                .key_path("id")
                .auto_increment(true)
                .build()?;
        }
        Ok(())
    })
    .build()
    .await?;

// List stores
let stores = db.object_store_names();

// Delete a database
Database::delete("mydb").await?;
```

### CRUD Operations

```rust
let store = db.transaction_on("users").rw().build()?.store("users")?;

// Create
store.add(&user).await?;                    // Fails if key exists
store.put(&user).await?;                    // Insert or update

// Read
let user: Option<User> = store.get(1).await?;
let all: Vec<User> = store.get_all().await?;
let count = store.count().await?;

// Update
store.put(&updated_user).await?;

// Delete
store.delete(1).await?;
store.clear().await?;
```

### Indexes

```rust
// Create an index during upgrade
ctx.create_object_store("users")
    .key_path("id")
    .index("email", "email")           // Regular index
    .unique_index("username", "username") // Unique constraint
    .multi_entry_index("tags", "tags") // For array fields
    .build()?;

// Query by index
let index = store.index("email")?;
let user: Option<User> = index.get("alice@example.com").await?;
let users: Vec<User> = index.get_all("alice@example.com").await?;
```

### Cursors

```rust
// Iterate all records
let mut cursor = store.open_cursor().await?;
while let Some(user) = cursor.next::<User>().await? {
    println!("{:?}", user);
}

// Collect into Vec
let users: Vec<User> = store.open_cursor().await?.collect().await?;

// Filter records
let admins = store.open_cursor().await?
    .filter(|u: &User| u.role == "admin")
    .await?;

// Find first match
let alice = store.open_cursor().await?
    .find(|u: &User| u.name == "Alice")
    .await?;

// Pagination
let page = store.open_cursor().await?
    .skip(20)
    .await?
    .take(10)
    .await?;

// Reverse iteration
let mut cursor = store
    .open_cursor_with_direction(CursorDirection::Prev)
    .await?;
```

### Key Ranges

```rust
// Query with ranges
let range = KeyRange::bound(10, 20, false, false)?; // [10, 20]
let users = store.get_all_in_range::<User>(&range).await?;

// Convenience methods
let range = KeyRange::gte(100)?;    // >= 100
let range = KeyRange::lt(50)?;      // < 50
let range = KeyRange::only(42)?;    // == 42

// Open cursor with range
let mut cursor = store.open_cursor_range(&range).await?;
```

### Transactions

```rust
// Read-only transaction (default)
let tx = db.transaction_on("users").build()?;

// Read-write transaction
let tx = db.transaction_on("users").rw().build()?;

// Multiple stores
let tx = db.transaction(&["users", "posts"]).rw().build()?;
let users = tx.store("users")?;
let posts = tx.store("posts")?;

// Commit (wait for completion)
tx.commit().await?;

// Abort
tx.abort()?;
```

## Error Handling

All errors are unified in `IdbError`:

```rust
use ferric_idb::{IdbError, IdbResult};

async fn example() -> IdbResult<()> {
    match store.get::<User>(1).await {
        Ok(Some(user)) => println!("Found: {:?}", user),
        Ok(None) => println!("Not found"),
        Err(IdbError::ConstraintError(msg)) => println!("Duplicate key: {}", msg),
        Err(IdbError::TransactionInactive) => println!("Transaction expired"),
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}
```

## Browser Support

IndexedDB is supported in all modern browsers:
- Chrome 23+
- Firefox 10+
- Safari 10+
- Edge 12+

## License

MIT

