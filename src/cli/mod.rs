use clap::Parser;
use sqlx::{query, Acquire};

use crate::Runner;

mod proj;

#[derive(Parser)]
pub enum Cmd {
    #[clap(subcommand)]
    Proj(proj::Proj),
    On(On),
    Off(Off),
    Cur(Cur),
}

#[derive(Parser)]
pub struct Off {}

#[derive(Parser)]
pub struct Cur {}

#[derive(Parser)]
pub struct On {
    project: String,
}

impl Runner for Cmd {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        match self {
            Cmd::Proj(proj) => proj.run(db).await,
            Cmd::On(on) => on.run(db).await,
            Cmd::Off(off) => off.run(db).await,
            Cmd::Cur(cur) => cur.run(db).await,
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

impl Runner for Cur {
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
