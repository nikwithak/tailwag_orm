use std::{collections::HashMap, sync::Arc};

use crate::{
    data_definition::{
        database_definition::DatabaseDefinition,
        table::{
            DatabaseTableDefinition, ForeignKeyConstraint, Identifier, TableColumn,
            TableConstraint, TableConstraintDetail,
        },
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

        // Build a map for quick lookup of after_tables, then compare each
        let mut after_columns: HashMap<&Identifier, &TableColumn> = after.columns.iter().collect();
        for (_, old_column) in &before.columns {
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
                        let old_is_nullable = old_column.constraints.iter().find(|c| match *c.detail {
                                // Allowed null unless NOT NULL
                                crate::data_definition::table::TableColumnConstraintDetail::NotNull => true,
                                _ => false,
                            }).is_none();
                        let new_is_nullable = new_column.constraints.iter().find(|c| match *c.detail {
                                crate::data_definition::table::TableColumnConstraintDetail::NotNull => true,
                                _ => false,
                            }).is_none();
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

                    if alter_column_actions.len() > 0 {
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
