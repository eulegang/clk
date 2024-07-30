use clap::Parser;
use eyre::ContextCompat as _;

use crate::cli::Cmd;
use sqlx::ConnectOptions;

mod cli;

trait Runner {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()>;
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Cmd::parse();

    let mut dir = dirs::data_dir().wrap_err("could not locate data dir")?;
    dir.push("clk");

    let _ = std::fs::create_dir_all(&dir);
    dir.push("clk.sqlite3");

    let mut db = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(dir)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true)
        .connect()
        .await?;

    sqlx::migrate!().run(&mut db).await?;

    args.run(&mut db).await?;

    Ok(())
}
