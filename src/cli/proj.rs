use clap::{Parser, Subcommand};
use sqlx::prelude::*;

use crate::Runner;

/// Manage projects
#[derive(Subcommand)]
pub enum Proj {
    Add(ProjAdd),
    Ls(ProjLs),
    Rm(ProjRm),
}

/// Create a project
#[derive(Parser)]
pub struct ProjAdd {
    /// The project name
    name: String,
}

/// Lists the projects
#[derive(Parser)]
pub struct ProjLs {}

/// Removes a project
#[derive(Parser)]
pub struct ProjRm {
    /// The project name
    name: String,
}

impl Runner for Proj {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        match self {
            Proj::Add(add) => add.run(db).await,
            Proj::Ls(ls) => ls.run(db).await,
            Proj::Rm(rm) => rm.run(db).await,
        }
    }
}

#[derive(FromRow)]
struct Project {
    name: String,
}

impl Runner for ProjLs {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        let projs = sqlx::query_as!(Project, "select name from Projects")
            .fetch_all(db)
            .await?;

        for proj in projs {
            println!("{}", proj.name);
        }

        Ok(())
    }
}

impl Runner for ProjAdd {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        sqlx::query!(r#"insert into Projects(name) values (?)"#, self.name)
            .execute(db)
            .await?;

        Ok(())
    }
}

impl Runner for ProjRm {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        sqlx::query!(r#"delete from  Projects where name = ?"#, self.name)
            .execute(db)
            .await?;

        Ok(())
    }
}
