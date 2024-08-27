use std::{collections::HashMap, sync::Arc};

use sqlx::{Pool, Postgres};

use crate::{
    data_definition::{
        database_definition::DatabaseDefinition,
        exp_data_system::{DataSystem, TableDef},
        table::{
            self, raw_data::TableDefinition, DatabaseTableDefinition, ForeignKeyConstraint,
            Identifier, TableColumn, TableConstraint, TableConstraintDetail,
        },
    },
    migration::{AlterColumn, AlterColumnAction, AlterTableAction},
    BuildSql,
};

use super::{AlterTable, CreateTable};

#[derive(Clone)]
pub enum MigrationAction {
    AlterTable(AlterTable),
    CreateTable(CreateTable),
    DropTable(Identifier),
}

impl BuildSql for MigrationAction {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        match self {
            MigrationAction::AlterTable(alter_table) => alter_table.build_sql(builder),
            MigrationAction::CreateTable(create_table) => create_table.build_sql(builder),
            MigrationAction::DropTable(table_ident) => {
                builder.push("DROP TABLE IF EXISTS ").push(&**table_ident);
            },
        };
    }
}

#[derive(Clone)]
pub struct Migration {
    pub actions: Vec<MigrationAction>,
}

impl Migration {
    pub async fn run(
        self,
        db_pool: &Pool<Postgres>,
    ) -> Result<(), crate::Error> {
        let mut builder = sqlx::QueryBuilder::new("");
        self.build_sql(&mut builder);
        builder.build().execute(db_pool).await?;

        Ok(())
    }
}

impl BuildSql for Migration {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        for alter_table in &self.actions {
            alter_table.build_sql(builder);
            builder.push("\n");
        }
    }
}

impl Migration {
    pub(crate) fn compare(
        before: Option<Vec<TableDef>>,
        after: Vec<TableDef>,
    ) -> Option<Self> {
        let mut actions: Vec<MigrationAction> = Vec::new();

        fn build_table_map(db_def: Vec<TableDef>) -> HashMap<Identifier, TableDef> {
            let map = db_def.iter().fold(HashMap::new(), |mut acc, table| {
                acc.insert(table.table_name(), table.clone());
                acc
            });
            map
        }

        if let Some(before) = before {
            // Build a map for quick lookup of after_tables, then compare each
            let mut after_tables = build_table_map(after);
            for table_before in before {
                match after_tables
                    .remove(&table_before.table_name())
                    .map(|after_table| Self::compare_tables(table_before.clone(), after_table))
                {
                    Some(None) => {
                        println!("NO CHANGES");
                        // No changes to the table
                        // Do nothing
                    },
                    Some(Some(mut table_diff)) => {
                        println!("[MODIFY TABLE] {}", table_before.table_name());
                        // println!("before: {:?}", table_before);
                        // Table has been modified
                        actions.append(&mut table_diff.actions);
                    },
                    None => {
                        println!("[DELETE TABLE] {}", table_before.table_name());
                        // Table was not found in new tables, meaning it was deleted
                        // TODO: At the end, compare any deleted/added tables to see if there was just some renaming done.
                        let action = MigrationAction::DropTable(table_before.table_name().clone());
                        actions.push(action);
                    },
                };
            }

            // Any remaining tables are new - add the CreateTable actions
            actions.append(
                &mut after_tables
                    .values()
                    .map(|table| MigrationAction::CreateTable(CreateTable::new(table.clone()))) //(*table).clone())))
                    .collect(),
            )
        } else {
            // New database - only creates!
            let mut create_table_actions = after
                .iter()
                .map(|t| MigrationAction::CreateTable(CreateTable::new(t.clone())))
                .collect();
            actions.append(&mut create_table_actions);
        }

        if !actions.is_empty() {
            Some(Self {
                actions,
            })
        } else {
            None
        }
    }

    /// Returns `Some<Migration>` representing the steps required to go from `before` to `after`, or None if the inputs are the same.
    ///
    /// # Arguments
    ///
    /// * `before` - The existing table definition, before the new changes are applied.
    /// * `after` - The new table definition, currently in use.
    ///
    /// # Returns
    ///
    fn compare_tables(
        before: TableDef,
        after: TableDef,
    ) -> Option<Self> {
        let mut actions = Vec::<AlterTableAction>::new();

        // Name changed
        if before.table_name() != after.table_name() {
            actions.push(AlterTableAction::Rename(after.table_name()));
        }

        // Build a map for quick lookup of after_tables, then compare each
        let mut after_columns: HashMap<&Identifier, &TableColumn> =
            after.columns().iter().collect();
        for old_column in before.columns().values() {
            match after_columns.remove(&old_column.column_name) {
                Some(new_column) => {
                    let mut alter_column_actions = Vec::new();
                    if !old_column.column_type.eq(&new_column.column_type) {
                        alter_column_actions
                            .push(AlterColumnAction::SetType(new_column.column_type.clone()));
                    }

                    // * NONNULL calculation - Compares `NotNull`
                    {
                        // Uggggh this is really hacky. Wanna clean this up later.
                        // Find the existence of a `NotNull` constraint. If it does *not* exist (`.is_none()`) then the field *is* nullable.
                        // A confusing mess of double negative magic going on here.
                        let old_is_nullable = !old_column.constraints.iter().any(|c| match *c.detail {
                                // Allowed null unless NOT NULL
                                crate::data_definition::table::TableColumnConstraintDetail::NotNull => true,
                                _ => false,
                            });
                        let new_is_nullable = !new_column.constraints.iter().any(|c| {
                            matches!(
                                *c.detail,
                                crate::data_definition::table::TableColumnConstraintDetail::NotNull
                            )
                        });
                        if old_is_nullable != new_is_nullable {
                            alter_column_actions
                                .push(AlterColumnAction::SetNullability(new_is_nullable));
                        }
                    }

                    // * TODO: Foreign Key Changes
                    // We compare the columns, and add the _TABLE'S_ FK constraint, because we can't add the constraint to column except at creation.
                    // TODO: Do another pass for the FK constraints.
                    {
                        let old_fk = old_column.constraints.iter().find_map(|c| match &*c.detail {
                            crate::data_definition::table::TableColumnConstraintDetail::References(fk) => Some(fk),
                            _ => None,
                        });
                        let new_fk = new_column.constraints.iter().find_map(|c| match &*c.detail {
                            crate::data_definition::table::TableColumnConstraintDetail::References(fk) => Some(fk),
                            _ => None,
                        });
                        match (old_fk, new_fk) {
                            (None, None) => None,
                            (None, Some(new_fk)) => {
                                // FK was added
                                Some(AlterTableAction::AddConstraint(TableConstraint {
                                    name: None, // TODO: Infer name. I might need to rework how these constraints work :facepalm:
                                    detail: Arc::new(TableConstraintDetail::ForeignKey(
                                        ForeignKeyConstraint {
                                            ref_table: new_fk.ref_table.clone(),
                                            ref_columns: vec![new_fk
                                                .ref_column
                                                .as_ref()
                                                .unwrap()
                                                .clone()],
                                            columns: vec![new_column.clone()],
                                            match_type: new_fk.match_type.clone(),
                                            on_delete_action: new_fk.on_delete_action.clone(),
                                            on_update_action: new_fk.on_update_action.clone(),
                                        },
                                    )),
                                }))
                            },
                            (Some(_), None) => todo!(), // DELETE constraint from column. Need a consistent way to name constraints in order to do this.
                            (Some(old_fk), Some(new_fk)) => {
                                if old_fk.ne(new_fk) {
                                    todo!("Support for updating FKs not yet implemented")
                                } else {
                                    None
                                }
                            }, // COMPARE constraints
                        };
                    }

                    if !alter_column_actions.is_empty() {
                        actions.push(AlterTableAction::AlterColumn(AlterColumn {
                            column_name: new_column.column_name.clone(),
                            actions: alter_column_actions,
                        }));
                    }
                },
                None => {
                    // Column Deleted
                    actions.push(AlterTableAction::DropColumn(old_column.column_name.clone()));
                },
            }
        }

        // Any remianing columns are new
        for column in after_columns.values() {
            actions.push(AlterTableAction::AddColumn((*column).clone()));
        }

        if !actions.is_empty() {
            Some(Self {
                actions: vec![MigrationAction::AlterTable(AlterTable {
                    table_name: after.table_name().clone(),
                    actions,
                })],
            })
        } else {
            None
        }
    }
}
