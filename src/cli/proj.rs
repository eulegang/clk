use clap::{Parser, Subcommand};
use sqlx::{prelude::*, query, query_as};

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
        let projs: Vec<Project> = query_as("select name from Projects").fetch_all(db).await?;

        for proj in projs {
            println!("{}", proj.name);
        }

        Ok(())
    }
}

impl Runner for ProjAdd {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        query(r#"insert into Projects(name) values (?)"#)
            .bind(self.name)
            .execute(db)
            .await?;

        Ok(())
    }
}

impl Runner for ProjRm {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        query(r#"delete from  Projects where name = ?"#)
            .bind(self.name)
            .execute(db)
            .await?;

        Ok(())
    }
}
