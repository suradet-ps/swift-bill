//! Tiberius TCP/TDS connection helper.

use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use swift_bill_core::DbConfig;

/// Alias for the concrete tiberius client type we use.
pub type TiberiusClient = Client<tokio_util::compat::Compat<TcpStream>>;

/// Open a TCP/TDS connection to the configured SQL Server.
///
/// # Errors
///
/// Returns a [`DbError`] if the TCP connection cannot be opened, the TLS
/// handshake fails, or the TDS login is rejected.
pub async fn connect(db_config: &DbConfig) -> Result<TiberiusClient, DbError> {
  let mut config = Config::new();
  config.host(&db_config.host);
  config.port(db_config.port);
  config.database(&db_config.database);
  config.authentication(AuthMethod::sql_server(
    &db_config.username,
    &db_config.password,
  ));
  config.trust_cert();

  let tcp = TcpStream::connect(config.get_addr())
    .await
    .map_err(|e| DbError::Tcp(e.to_string()))?;

  tcp
    .set_nodelay(true)
    .map_err(|e| DbError::Tcp(e.to_string()))?;

  let client = Client::connect(config, tcp.compat_write())
    .await
    .map_err(|e| DbError::Tds(e.to_string()))?;

  Ok(client)
}

/// Database connection error.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
  #[error("TCP connection failed: {0}")]
  Tcp(String),
  #[error("TDS connection failed: {0}")]
  Tds(String),
}
