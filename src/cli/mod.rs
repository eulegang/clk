use clap::Parser;
use eyre::{Context as _, ContextCompat as _};
use sqlx::{query, Acquire, Column, Row, Value, ValueRef};

use crate::Runner;

mod proj;

/// Time tracking cli app
#[derive(Parser)]
pub enum Cmd {
    #[clap(subcommand)]
    Proj(proj::Proj),
    On(On),
    Off(Off),
    Status(Status),
    Report(Report),
}

/// Stops the current work entry
#[derive(Parser)]
pub struct Off {}

/// Checks the current work entry
#[derive(Parser)]
pub struct Status {}

/// Starts a work entry
#[derive(Parser)]
pub struct On {
    project: String,
}

/// Runs a report
#[derive(Parser)]
pub struct Report {
    report: String,

    args: Vec<String>,
}

impl Runner for Cmd {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        match self {
            Cmd::Proj(proj) => proj.run(db).await,
            Cmd::On(on) => on.run(db).await,
            Cmd::Off(off) => off.run(db).await,
            Cmd::Status(cur) => cur.run(db).await,
            Cmd::Report(report) => report.run(db).await,
        }
    }
}

impl Runner for On {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        let mut lock = db.begin().await?;
        let count = query!(r#"select count(*) as cnt from Entries where end is null"#)
            .fetch_one(lock.as_mut())
            .await?;

        if count.cnt > 0 {
            eprintln!("Currently on the clock");
            std::process::exit(1);
        }

        let res = query!(r#"insert into Entries (project_id, start) select id project_id, unixepoch() start from Projects where name = ?"#, self.project).execute(lock.as_mut()).await?;

        if dbg!(res.rows_affected()) != 1 {
            lock.rollback().await?;

            eprintln!("failed to create record");
            std::process::exit(1);
        } else {
            lock.commit().await?;
        }

        Ok(())
    }
}

impl Runner for Off {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        let mut lock = db.begin().await?;
        let count = query!(r#"select count(*) as cnt from Entries where end is null"#)
            .fetch_one(lock.as_mut())
            .await?;

        if count.cnt == 0 {
            eprintln!("Currently off the clock");
            std::process::exit(1);
        } else if count.cnt > 1 {
            eprintln!("Open entry invariant violated");
            std::process::exit(1);
        }

        query!(r#"update Entries set end = unixepoch() where end is null"#)
            .execute(lock.as_mut())
            .await?;

        lock.commit().await?;

        Ok(())
    }
}

impl Runner for Status {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        let row = query!(r#"select name, unixepoch() now, start from Entries inner join Projects on Entries.project_id = Projects.id where end is null"#).fetch_optional(db).await?;

        if let Some(row) = row {
            let dur = calc(row.now, row.start);
            println!("{} {}", row.name, dur);
        } else {
            println!("off the clock");
        }

        Ok(())
    }
}

impl Runner for Report {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        let mut s = dirs::config_dir().wrap_err("expected to find config dir")?;
        s.push("clk");
        s.push("reports");

        s.push(&format!("{}.sql", self.report));

        let sql = std::fs::read_to_string(&s).wrap_err("no report exists")?;

        let mut q = sqlx::query(&sql);

        for arg in self.args {
            q = q.bind(arg);
        }

        let results = q.fetch_all(db).await?;

        let stdout = std::io::stdout();
        let mut writer = csv::Writer::from_writer(stdout.lock());

        if let Some(row) = results.first() {
            for col in row.columns() {
                writer.write_field(col.name())?;
            }

            writer.write_record(None::<&[u8]>)?;
        }

        for row in results {
            for i in 0..row.len() {
                let val = row.try_get_raw(i)?.to_owned();

                if val.is_null() {
                    writer.write_field("null")?;
                } else if let Ok(i) = val.try_decode::<i64>() {
                    writer.write_field(i.to_string())?;
                } else if let Ok(b) = val.try_decode::<bool>() {
                    writer.write_field(b.to_string())?;
                } else if let Ok(s) = val.try_decode::<String>() {
                    writer.write_field(s)?;
                } else if let Ok(b) = val.try_decode::<Vec<u8>>() {
                    writer.write_field(b)?;
                } else if let Ok(f) = val.try_decode::<f64>() {
                    writer.write_field(f.to_string())?;
                } else {
                    eprintln!("failed to decode");
                }
            }

            writer.write_record(None::<&[u8]>)?;
        }

        Ok(())
    }
}

fn calc(now: i64, start: i64) -> String {
    let mut dur = now - start;
    let seconds = dur % 60;
    dur /= 60;

    if dur == 0 {
        return format!("{}s", seconds);
    }

    let minutes = dur % 60;
    dur /= 60;

    if dur == 0 {
        return format!("{}m {}s", minutes, seconds);
    }

    let hours = dur % 24;
    dur /= 24;

    if dur == 0 {
        return format!("{}h {}m {}s", hours, minutes, seconds);
    }

    let days = dur;

    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
}
