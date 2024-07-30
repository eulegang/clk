use clap::{Parser, Subcommand};
use sqlx::prelude::*;

use crate::Runner;

/// Manage projects
#[derive(Subcommand)]
pub enum Proj {
    Create(ProjCreate),
    List(ProjList),
    Remove(ProjRemove),
}

/// Create a project
#[derive(Parser)]
pub struct ProjCreate {
    /// The project name
    name: String,
}

/// Lists the projects
#[derive(Parser)]
pub struct ProjList {}

/// Removes a project
#[derive(Parser)]
pub struct ProjRemove {
    /// The project name
    name: String,
}

impl Runner for Proj {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        match self {
            Proj::Create(create) => create.run(db).await,
            Proj::List(list) => list.run(db).await,
            Proj::Remove(remove) => remove.run(db).await,
        }
    }
}

#[derive(FromRow)]
struct Project {
    name: String,
}

impl Runner for ProjList {
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

impl Runner for ProjCreate {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        sqlx::query!(r#"insert into Projects(name) values (?)"#, self.name)
            .execute(db)
            .await?;

        Ok(())
    }
}

impl Runner for ProjRemove {
    async fn run(self, db: &mut sqlx::sqlite::SqliteConnection) -> eyre::Result<()> {
        sqlx::query!(r#"delete from  Projects where name = ?"#, self.name)
            .execute(db)
            .await?;

        Ok(())
    }
}
