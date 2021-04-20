use crate::adaptor::DbAdaptor;
use crate::errors::{Error, Result};
use crate::migration::{Migration, MigrationBuilder};
use mysql::{prelude::Queryable as _, TxOpts};

impl DbAdaptor for mysql::Conn {
    fn init_up_sql(&self) -> &'static str {
        INIT_UP_SQL
    }

    fn init_down_sql(&self) -> &'static str {
        INIT_DOWN_SQL
    }

    fn load_migrations(&mut self) -> Result<Vec<Migration>> {
        let mut migrations = Vec::new();
        let sql = "
            SELECT name, hash, down_sql
            FROM movine_migrations
            ORDER BY created_at DESC;
        ";
        let stmt = self.prep(&sql)?;
        let rows = self.exec::<(String, String, String), _, _>(stmt, ())?;

        for row in rows {
            let name: String = row.0;
            let hash: String = row.1;
            let down_sql: String = row.2;
            let migration = MigrationBuilder::new()
                .compound_name(&name)
                .hash(&hash)
                .down_sql(&down_sql)
                .build()?;
            migrations.push(migration);
        }
        Ok(migrations)
    }

    fn run_up_migration(&mut self, migration: &Migration) -> Result<()> {
        let name = &migration.name;
        let hash = migration.hash.as_ref().ok_or_else(|| Error::BadMigration)?;
        let up_sql = migration
            .up_sql
            .as_ref()
            .ok_or_else(|| Error::BadMigration)?;
        let empty_string = "".to_string();
        let down_sql = migration.down_sql.as_ref().unwrap_or_else(|| &empty_string);

        let mut transaction = self.start_transaction(TxOpts::default())?;
        transaction.query_drop(&up_sql)?;
        transaction.exec_drop(LOG_UP_MIGRATION, vec![&name, &hash, &down_sql])?;
        transaction.commit()?;
        Ok(())
    }

    fn run_down_migration(&mut self, migration: &Migration) -> Result<()> {
        let name = &migration.name;
        let down_sql = migration
            .down_sql
            .as_ref()
            .ok_or_else(|| Error::BadMigration)?;

        let mut transaction = self.start_transaction(TxOpts::default())?;
        transaction.query_drop(&down_sql)?;
        transaction.exec_drop(LOG_DOWN_MIGRATION, vec![&name])?;
        transaction.commit()?;
        Ok(())
    }
}

pub const LOG_UP_MIGRATION: &str = "\
INSERT INTO movine_migrations (name, hash, down_sql)
VALUES (?, ?, ?);
";

pub const LOG_DOWN_MIGRATION: &str = "\
DELETE FROM movine_migrations
WHERE name = ?;
";

pub const INIT_UP_SQL: &str = "\
CREATE TABLE movine_migrations (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    name TEXT NOT NULL,
    hash TEXT NOT NULL,
    down_sql TEXT
);
";

pub const INIT_DOWN_SQL: &str = "\
DROP TABLE movine_migrations;
";
