use std::collections::HashMap;

use crate::{
    database_definition::{
        database_definition::DatabaseDefinition,
        table_definition::{DatabaseTableDefinition, Identifier, TableColumn},
    },
    migration::{AlterColumn, AlterColumnAction, AlterTableAction},
    AsSql,
};

use super::{AlterTable, CreateTable};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MigrationAction {
    AlterTable(AlterTable),
    CreateTable(CreateTable),
    DropTable(Identifier),
}

impl AsSql for MigrationAction {
    fn as_sql(&self) -> String {
        match self {
            MigrationAction::AlterTable(alter_table) => alter_table.as_sql(),
            MigrationAction::CreateTable(create_table) => create_table.as_sql(),
            MigrationAction::DropTable(table_ident) => {
                format!("DROP TABLE IF EXISTS {};", &table_ident)
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Migration {
    pub actions: Vec<MigrationAction>,
}

impl AsSql for Migration {
    fn as_sql(&self) -> String {
        let mut sql_statments = Vec::new();
        for alter_table in &self.actions {
            let statement = alter_table.as_sql();

            sql_statments.push(statement);
        }

        let statement = sql_statments.join("\n");
        statement
    }
}

impl Migration {
    pub fn compare(
        before: Option<&DatabaseDefinition>,
        after: &DatabaseDefinition,
    ) -> Option<Self> {
        let mut actions: Vec<MigrationAction> = Vec::new();

        fn build_table_map(
            db_def: &DatabaseDefinition
        ) -> HashMap<&Identifier, &DatabaseTableDefinition> {
            let map = db_def.tables.iter().fold(HashMap::new(), |mut acc, table| {
                acc.insert(&table.table_name, table);
                acc
            });
            map
        }

        if let Some(before) = before {
            // Build a map for quick lookup of after_tables, then compare each
            let mut after_tables = build_table_map(after);
            for table_before in &before.tables {
                match after_tables
                    .remove(&table_before.table_name)
                    .map(|after_table| Self::compare_tables(table_before, after_table))
                {
                    Some(None) => {
                        println!("NO CHANGES");
                        // No changes to the table
                        // Do nothing
                    },
                    Some(Some(mut table_diff)) => {
                        println!("[MODIFY TABLE]");
                        println!("before: {:?}", table_before);
                        // Table has been modified
                        actions.append(&mut table_diff.actions);
                    },
                    None => {
                        println!("[DELETE TABLE] {:?}", table_before);
                        // Table was not found in new tables, meaning it was deleted
                        // TODO: At the end, compare any deleted/added tables to see if there was just some renaming done.
                        let action = MigrationAction::DropTable(table_before.table_name.clone());
                        actions.push(action);
                    },
                };
            }

            // Any remaining tables are new - add the CreateTable actions
            actions.append(
                &mut after_tables
                    .values()
                    .map(|table| MigrationAction::CreateTable(CreateTable::new((*table).clone())))
                    .collect(),
            )
        } else {
            // New database - only creates!
            let mut create_table_actions = after
                .tables
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
        before: &DatabaseTableDefinition,
        after: &DatabaseTableDefinition,
    ) -> Option<Self> {
        let mut actions = Vec::<AlterTableAction>::new();

        // Name changed
        if !(before.table_name == after.table_name) {
            actions.push(AlterTableAction::Rename(after.table_name.clone()));
        }

        fn build_column_map(table: &DatabaseTableDefinition) -> HashMap<&Identifier, &TableColumn> {
            let map = table.columns.iter().fold(HashMap::new(), |mut acc, column| {
                acc.insert(&column.column_name, column);
                acc
            });
            map
        }

        // Build a map for quick lookup of after_tables, then compare each
        let mut after_columns = build_column_map(after);
        for old in &before.columns {
            match after_columns.remove(&old.column_name) {
                Some(new) => {
                    let mut alter_column_actions = Vec::new();
                    if !old.column_type.eq(&new.column_type) {
                        alter_column_actions
                            .push(AlterColumnAction::SetType(new.column_type.clone()));
                    }

                    // * NONNULL calculation - Compares `NotNull`
                    {
                        // Uggggh this is really hacky. Wanna clean this up later.
                        // Find the existence of a `NotNull` constraint. If it does *not* exist (`.is_none()`) then the field *is* nullable.
                        // A confusing mess of double negative magic going on here.
                        let old_is_nullable = old.constraints.iter().find(|c| match *c.detail {
                                // Allowed null unless NOT NULL
                                crate::database_definition::table_definition::TableColumnConstraintDetail::NotNull => true,
                                _ => false,
                            }).is_none();
                        let new_is_nullable = new.constraints.iter().find(|c| match *c.detail {
                                crate::database_definition::table_definition::TableColumnConstraintDetail::NotNull => true,
                                _ => false,
                            }).is_none();
                        if old_is_nullable != new_is_nullable {
                            alter_column_actions
                                .push(AlterColumnAction::SetNullability(new_is_nullable));
                        }
                    }

                    // * TODO: Foreign Key Changes
                    // {
                    //     let old_fks = old.constraints.iter().find(|c| match *c.detail {
                    //         // TODO: DRY this out a bit so that TableConstraintDetail /
                    //         crate::database_definition::table_definition::TableColumnConstraintDetail::NotNull => todo!(),
                    //         crate::database_definition::table_definition::TableColumnConstraintDetail::Null => todo!(),
                    //         crate::database_definition::table_definition::TableColumnConstraintDetail::Unique(_) => todo!(),
                    //         crate::database_definition::table_definition::TableColumnConstraintDetail::PrimaryKey(_) => todo!(),
                    //         crate::database_definition::table_definition::TableColumnConstraintDetail::References(_) => todo!(),
                    //     });
                    // }

                    if alter_column_actions.len() > 0 {
                        actions.push(AlterTableAction::AlterColumn(AlterColumn {
                            column_name: new.column_name.clone(),
                            actions: alter_column_actions,
                        }));
                    }
                },
                None => {
                    // Column Deleted
                    actions.push(AlterTableAction::DropColumn(old.column_name.clone()));
                },
            }
        }

        // Any remianing columns are new
        for column in after_columns.values() {
            actions.push(AlterTableAction::AddColumn((*column).clone()));
        }

        if actions.len() > 0 {
            Some(Self {
                actions: vec![MigrationAction::AlterTable(AlterTable {
                    table_name: after.table_name.clone(),
                    actions,
                })],
            })
        } else {
            None
        }
    }
}
