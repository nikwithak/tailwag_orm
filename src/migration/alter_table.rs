use crate::{
    data_definition::table::{DatabaseColumnType, Identifier, TableColumn, TableConstraint},
    AsSql,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AlterTableAction {
    Rename(Identifier),
    AddColumn(TableColumn),   // TODO
    DropColumn(Identifier),   // TODO
    AlterColumn(AlterColumn), // TODO
    // TODO: Add the rest of the actions.
    // Ref: https://www.postgresql.org/docs/current/sql-altertable.html
    AddConstraint(TableConstraint),
    AlterConstraint(),
    DropConstraint(TableConstraint),
}

impl AsSql for AlterTableAction {
    fn as_sql(&self) -> String {
        type E = AlterTableAction;
        match self {
            E::Rename(ident) => format!("RENAME TO {}", ident),
            E::AddColumn(table_column) => {
                format!("ADD COLUMN IF NOT EXISTS {}", table_column.as_sql())
            },
            E::DropColumn(ident) => format!("DROP COLUMN IF EXISTS {}", ident),
            E::AlterColumn(alter_column) => alter_column.as_sql(),
            E::AddConstraint(constraint) => format!("ADD {}", &constraint.as_sql()),
            E::AlterConstraint() => todo!(),
            E::DropConstraint(constraint) => {
                format!(
                    "DROP CONSTRAINT IF EXISTS {}",
                    &constraint
                        .name
                        .as_ref()
                        .map(|c| c.to_string())
                        .unwrap_or("UNNAMED_CONSTRAINT".to_string())
                )
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlterTable {
    pub table_name: Identifier,
    pub actions: Vec<AlterTableAction>,
}

impl AsSql for AlterTable {
    fn as_sql(&self) -> String {
        let statements = self
            .actions
            .iter()
            .map(|action| action.as_sql())
            .collect::<Vec<String>>()
            .iter()
            .map(|action_sql| format!("ALTER TABLE IF EXISTS {} {};", self.table_name, &action_sql))
            .collect::<Vec<String>>()
            .join("\n");

        statements
    }
}

// TODO: Part of _SetStorage() requirement
// enum StorageType {
//     Plain,
//     External,
//     Extended,
//     Main,
// }

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AlterColumnAction {
    SetType(DatabaseColumnType), // TODO: Look at the other options here
    SetNullability(bool),        // True if nullable, False if not.
                                 // _DropExpression,            // TODO: Unsupported yet
                                 // _AddGenerated(),            // TODO: Unsupported yet
                                 // _SetGenerated(),            // TODO: Unsupported yet
                                 // _DropIdentity,              // TODO: Unsupported yet Always use IF EXISTS
                                 // _SetStatistics(i64),        // TODO: Unsupported yet
                                 // _SetAttribute(),            // TODO: Unsupported yet
                                 // _Reset(),                   // TODO: Unsupported yet
                                 // _SetStorage(StorageType),   // TODO: Unsupported yet
                                 // _SetCompression(_CompressionMethod), // TODO: Unsupported yet
}

impl AsSql for AlterColumnAction {
    fn as_sql(&self) -> String {
        match self {
            AlterColumnAction::SetType(t) => format!("TYPE {}", t.as_str()),
            AlterColumnAction::SetNullability(nullable) => {
                #[rustfmt::skip]
                let verb = if *nullable { "DROP" } else { "SET" };
                format!("{} NOT NULL", verb)
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlterColumn {
    pub column_name: Identifier,
    pub actions: Vec<AlterColumnAction>,
}

impl AsSql for AlterColumn {
    fn as_sql(&self) -> String {
        let actions_sql = self
            .actions
            .iter()
            .map(|action| action.as_sql())
            .collect::<Vec<String>>()
            .iter()
            .map(|action| format!("ALTER COLUMN {} {}", &self.column_name, &action))
            .collect::<Vec<String>>()
            .join(", ");

        actions_sql
    }
}
