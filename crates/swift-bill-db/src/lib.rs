//! Database access for the INVS SQL Server.
//!
//! Uses `tiberius` over plain TCP/TDS — no ODBC driver required. All
//! public APIs are async and return `Result<_, DbError>` so callers can
//! format their own user-facing error strings.

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod connect;
pub mod invoices;

pub use connect::{TiberiusClient, connect};
pub use invoices::fetch_invoices;
