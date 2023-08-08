use crate::{
    database_definition::{
        database_definition::DatabaseDefinition,
        migration::{AlterColumn, AlterColumnAction, AlterTableAction},
        table_definition::{DatabaseTableDefinition, Identifier, TableColumn},
    },
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
        if let Some(before) = before {
            let mut before_tables_sorted =
                before.tables.iter().collect::<Vec<&DatabaseTableDefinition>>();
            before_tables_sorted.sort_by(|l, r| l.table_name.cmp(&r.table_name));
            let mut after_tables_sorted =
                after.tables.iter().collect::<Vec<&DatabaseTableDefinition>>();
            after_tables_sorted.sort_by(|l, r| l.table_name.cmp(&r.table_name));

            let mut old_sorted_iter = before_tables_sorted.iter();
            let mut new_sorted_iter = after_tables_sorted.iter();

            // Now that they're sorted, walk both and build actions based on conflicts. Same approach as compare_tables below.
            // TODO: DRY out the walking pattern. It'll also save time if I move it from a sorting approach to a bucketed approach to improve CPU time
            // Sorting = O(nlog(n)), Bucketing would be O(n). Might not even need that, could change table def to be a HM
            // Not *very* important for this use, since it's meant to be run only during build & deploy, not at runtime.
            let mut old_table = old_sorted_iter.next();
            let mut new_table = new_sorted_iter.next();

            while old_table.is_some() || new_table.is_some() {
                match (old_table, new_table) {
                    (None, Some(new)) => {
                        // new = an entirely new table
                        actions
                            .push(MigrationAction::CreateTable(CreateTable::new((*new).clone())));
                        println!("new: {:?}", &new_table);
                        println!("old: {:?}", &old_table);
                        new_table = new_sorted_iter.next();
                    },
                    (Some(old), None) => {
                        // old = removed (we never found a match)
                        actions.push(MigrationAction::DropTable(old.table_name.clone()));
                        println!("new: {:?}", &new_table);
                        println!("old: {:?}", &old_table);
                        old_table = old_sorted_iter.next();
                    },
                    (Some(old), Some(new)) => {
                        match old.table_name.cmp(&new.table_name) {
                            std::cmp::Ordering::Less => {
                                // Old table comes first, and has been deleted.
                                actions.push(MigrationAction::DropTable(old.table_name.clone()));
                                // TODO: Remove all the println!s here. Maybe convert them to debug? Definitely move it to top of thing so it's only called once.
                                println!("new: {:?}", &new_table);
                                println!("old: {:?}", &old_table);
                                old_table = old_sorted_iter.next();
                            },
                            std::cmp::Ordering::Equal => {
                                if let Some(mut migration) = Self::compare_tables(old, new) {
                                    // Same table with modifications - no action if both tables aren't there.
                                    actions.append(&mut migration.actions);
                                }
                                old_table = old_sorted_iter.next();
                                new_table = new_sorted_iter.next();
                            },
                            std::cmp::Ordering::Greater => {
                                // New table!
                                actions.push(MigrationAction::CreateTable(CreateTable::new(
                                    (*new).clone(),
                                )));
                                println!("new: {:?}", &new_table);
                                println!("old: {:?}", &old_table);
                                new_table = new_sorted_iter.next();
                            },
                        }
                        println!("new: {:?}", &new_table);
                        println!("old: {:?}", &old_table);
                    },
                    (None, None) => panic!("Should not ever reach here"),
                }
            }
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

        // Figure out added / removed / changed columns
        // First sort them, so we can iterate in one pass.
        let mut before_columns_sorted = before.columns.iter().collect::<Vec<&TableColumn>>();
        before_columns_sorted.sort_by(|l, r| l.column_name.cmp(&r.column_name));
        let mut after_columns_sorted = after.columns.iter().collect::<Vec<&TableColumn>>();
        after_columns_sorted.sort_by(|l, r| l.column_name.cmp(&r.column_name));

        for c in &before_columns_sorted {
            println!("{:?}", c);
        }
        println!("=========================");
        for c in &after_columns_sorted {
            println!("{:?}", c);
        }

        // Second, iterate through both. Any out-of-sync issues indicate a column only in that set.
        let mut old_sorted_iter = before_columns_sorted.into_iter().peekable();
        let mut new_sorted_iter = after_columns_sorted.into_iter().peekable();

        let mut old_column = old_sorted_iter.next();
        let mut new_column = new_sorted_iter.next();

        while old_column.is_some() || new_column.is_some() {
            match (old_column, new_column) {
                (None, Some(new)) => {
                    // new = an entirely new column
                    actions.push(AlterTableAction::AddColumn(new.clone()));
                    println!("    new: {:?}", &new_column);
                    println!("    old: {:?}", &old_column);
                    new_column = new_sorted_iter.next();
                },
                (Some(old), None) => {
                    // old = removed (we never found a match)
                    actions.push(AlterTableAction::DropColumn(old.column_name.clone()));
                    println!("    new: {:?}", &new_column);
                    println!("    old: {:?}", &old_column);
                    old_column = old_sorted_iter.next();
                },
                (Some(old), Some(new)) => match old.column_name.cmp(&new.column_name) {
                    std::cmp::Ordering::Less => {
                        println!("    new: {:?}", &new_column);
                        println!("    old: {:?}", &old_column);
                        actions.push(AlterTableAction::DropColumn(old.column_name.clone()));
                        old_column = old_sorted_iter.next();
                    },
                    std::cmp::Ordering::Greater => {
                        println!("    new: {:?}", &new_column);
                        println!("    old: {:?}", &old_column);
                        actions.push(AlterTableAction::AddColumn(new.clone()));
                        new_column = new_sorted_iter.next();
                    },
                    std::cmp::Ordering::Equal => {
                        println!("    new: {:?}", &new_column);
                        println!("    old: {:?}", &old_column);

                        // TODO move this logic into TableColumn? Or AlterColumnAction?
                        let mut alter_column_actions = Vec::new();
                        if !old.column_type.eq(&new.column_type) {
                            alter_column_actions
                                .push(AlterColumnAction::SetType(new.column_type.clone()));
                        }
                        if old.is_nullable != new.is_nullable {
                            alter_column_actions
                                .push(AlterColumnAction::SetNullability(new.is_nullable));
                        }

                        if alter_column_actions.len() > 0 {
                            actions.push(AlterTableAction::AlterColumn(AlterColumn {
                                column_name: new.column_name.clone(),
                                actions: alter_column_actions,
                            }));
                        }

                        new_column = new_sorted_iter.next();
                        old_column = old_sorted_iter.next();
                    },
                },
                (None, None) => panic!("Should not ever reach here"),
            }
        }

        if actions.len() > 0 {
            println!("=====   RESULT   =====");
            println!("{:?}", &actions);
            println!("===== END RESULT =====");
            Some(Self {
                actions: vec![MigrationAction::AlterTable(AlterTable {
                    actions,
                    table_name: before.table_name.clone(),
                })],
            })
        } else {
            None
        }
    }
}
