mod alter_table;
mod create_table;
mod migration;

pub use alter_table::*;
pub use create_table::*;
// #[cfg(migrations)]
pub use migration::*;

#[cfg(test)]
mod tests {
    use create_table::CreateTable;

    use crate::{
        database_definition::{
            database_definition::DatabaseDefinition,
            migration::{
                create_table, AlterColumn, AlterColumnAction, AlterTable, AlterTableAction,
                MigrationAction,
            },
            table_definition::{
                DatabaseColumnType, DatabaseTableDefinition, Identifier, TableColumn,
            },
        },
        AsSql,
    };

    use super::Migration;

    fn table_2() -> DatabaseTableDefinition {
        DatabaseTableDefinition {
            table_name: Identifier::new("table_2".to_string()).unwrap(),
            columns: vec![
                TableColumn {
                    parent_table_name: Identifier::new("table_2".to_string()).unwrap(),
                    column_name: Identifier::new("string_nullable".to_string()).unwrap(),
                    column_type: DatabaseColumnType::String,
                    is_primary_key: false,
                    is_nullable: true,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_2".to_string()).unwrap(),
                    column_name: Identifier::new("bool".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Boolean,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_2".to_string()).unwrap(),
                    column_name: Identifier::new("int".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Int,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_2".to_string()).unwrap(),
                    column_name: Identifier::new("float".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Float,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_2".to_string()).unwrap(),
                    column_name: Identifier::new("timestamp".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Timestamp,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_2".to_string()).unwrap(),
                    column_name: Identifier::new("uuid".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Uuid,
                    is_primary_key: false,
                    is_nullable: false,
                },
            ],
        }
    }

    fn table_1() -> DatabaseTableDefinition {
        DatabaseTableDefinition {
            table_name: Identifier::new("table_1".to_string()).unwrap(),
            columns: vec![
                TableColumn {
                    parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                    column_name: Identifier::new("string_nullable".to_string()).unwrap(),
                    column_type: DatabaseColumnType::String,
                    is_primary_key: false,
                    is_nullable: true,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                    column_name: Identifier::new("bool".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Boolean,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                    column_name: Identifier::new("int".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Int,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                    column_name: Identifier::new("float".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Float,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                    column_name: Identifier::new("timestamp".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Timestamp,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                    column_name: Identifier::new("uuid".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Uuid,
                    is_primary_key: false,
                    is_nullable: false,
                },
            ],
        }
    }

    fn table_3() -> DatabaseTableDefinition {
        DatabaseTableDefinition {
            table_name: Identifier::new("table_3".to_string()).unwrap(),
            columns: vec![
                TableColumn {
                    parent_table_name: Identifier::new("table_3".to_string()).unwrap(),
                    column_name: Identifier::new("created_at".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Timestamp,
                    is_primary_key: false,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_3".to_string()).unwrap(),
                    column_name: Identifier::new("id".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Uuid,
                    is_primary_key: true,
                    is_nullable: false,
                },
            ],
        }
    }

    fn table_4() -> DatabaseTableDefinition {
        DatabaseTableDefinition {
            table_name: Identifier::new("table_4".to_string()).unwrap(),
            columns: vec![
                TableColumn {
                    parent_table_name: Identifier::new("table_4".to_string()).unwrap(),
                    column_name: Identifier::new("id".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Uuid,
                    is_primary_key: true,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("table_4".to_string()).unwrap(),
                    column_name: Identifier::new("created_at".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Timestamp,
                    is_primary_key: false,
                    is_nullable: false,
                },
            ],
        }
    }

    #[test]
    fn as_sql_generates_sql_script() {
        // Arrange
        let migration = Migration {
            actions: vec![MigrationAction::AlterTable(AlterTable {
                table_name: Identifier::new("table_1".to_string()).unwrap(),
                actions: vec![
                    AlterTableAction::AlterColumn(AlterColumn {
                        column_name: Identifier::new("bool".to_string()).unwrap(),
                        actions: vec![
                            AlterColumnAction::SetType(DatabaseColumnType::String),
                            AlterColumnAction::SetNullability(true),
                        ],
                    }),
                    AlterTableAction::AlterColumn(AlterColumn {
                        column_name: Identifier::new("int".to_string()).unwrap(),
                        actions: vec![AlterColumnAction::SetType(DatabaseColumnType::Float)],
                    }),
                    AlterTableAction::AddColumn(TableColumn {
                        parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                        column_name: Identifier::new("new_column".to_string()).unwrap(),
                        column_type: DatabaseColumnType::String,
                        is_primary_key: false,
                        is_nullable: false,
                    }),
                    AlterTableAction::AlterColumn(AlterColumn {
                        column_name: Identifier::new("string_nullable".to_string()).unwrap(),
                        actions: vec![AlterColumnAction::SetNullability(false)],
                    }),
                    AlterTableAction::DropColumn(Identifier::new("timestamp".to_string()).unwrap()),
                ],
            })],
        };

        // Act
        let result_sql = migration.as_sql();

        // Assert
        // NOTE: This tests is a little finicky - does not account for different whitespace.
        //       This should be fine, but has room for improvement.
        let mut queries = result_sql.split("\n").collect::<Vec<&str>>();
        let mut expected_queries: Vec<&str> = vec![
            "ALTER TABLE IF EXISTS table_1 ALTER COLUMN bool TYPE VARCHAR, ALTER COLUMN bool DROP NOT NULL;",
            "ALTER TABLE IF EXISTS table_1 ALTER COLUMN int TYPE FLOAT;",
            "ALTER TABLE IF EXISTS table_1 ADD COLUMN IF NOT EXISTS  new_column VARCHAR  NOT NULL ;",
            "ALTER TABLE IF EXISTS table_1 ALTER COLUMN string_nullable SET NOT NULL;",
            "ALTER TABLE IF EXISTS table_1 DROP COLUMN IF EXISTS timestamp;",
        ];

        while !queries.is_empty() && !expected_queries.is_empty() {
            assert_eq!(queries.pop(), expected_queries.pop());
        }

        assert!(
            queries.is_empty() && expected_queries.is_empty(),
            "Number of queries did not match."
        );
    }

    #[test]
    fn compare_tables_new_database() {
        let before = None;
        let after = DatabaseDefinition::new("my_database".to_string(), vec![table_1(), table_2()]);

        let migration = Migration::compare(before.as_ref(), &after).unwrap();

        assert_eq!(
            migration,
            Migration {
                actions: vec![
                    MigrationAction::CreateTable(CreateTable::new(table_1())),
                    MigrationAction::CreateTable(CreateTable::new(table_2())),
                ],
            }
        );
    }

    #[test]
    fn compare_tables_no_diff() {
        let before = DatabaseDefinition::new("my_database".to_string(), vec![table_1(), table_2()]);
        let after = DatabaseDefinition::new("my_database".to_string(), vec![table_1(), table_2()]);

        // Act
        let migration = Migration::compare(Some(&before), &after);

        // Assert
        assert_eq!(migration, None);
    }

    #[test]
    fn compare_tables_builds_diff() {
        // Arrange
        let mut after_t1 = table_1();
        after_t1
            .columns
            .iter_mut()
            .find(|c| c.column_name.value().eq("int"))
            // Tests Type changes
            .map(|c| c.column_type = DatabaseColumnType::Float);
        after_t1
            .columns
            .iter_mut()
            .find(|c| c.column_name.value().eq("string_nullable"))
            .map(|c| {
                // Tests Nullability changes
                c.is_nullable = false;
            });
        after_t1.columns.iter_mut().find(|c| c.column_name.value().eq("bool")).map(|c| {
            // Tests a mix of the two changes
            c.column_type = DatabaseColumnType::String;
            c.is_nullable = true;
        });
        after_t1.columns.push(TableColumn {
            parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
            column_name: Identifier::new("new_column".to_string()).unwrap(),
            column_type: DatabaseColumnType::String,
            is_primary_key: false,
            is_nullable: false,
        });
        // Delete "timestamp" column
        after_t1.columns = after_t1
            .columns
            .into_iter()
            .filter(|c| !c.column_name.value().eq("timestamp"))
            .collect();

        let mut after_t4 = table_4();
        after_t4.columns.push(TableColumn {
            parent_table_name: Identifier::new("table_4".to_string()).unwrap(),
            column_name: Identifier::new("updated_at".to_string()).unwrap(),
            column_type: DatabaseColumnType::Timestamp,
            is_primary_key: false,
            is_nullable: true,
        });

        let before = DatabaseDefinition::new(
            "my_new_database".to_string(),
            vec![table_1(), table_2(), table_4()],
        );
        let after = DatabaseDefinition::new(
            "my_new_database".to_string(),
            vec![after_t1, table_3(), after_t4],
        );
        // Act
        let migration = Migration::compare(Some(&before), &after).unwrap();

        // Assert
        assert_eq!(
            migration,
            Migration {
                actions: vec![
                    MigrationAction::AlterTable(AlterTable {
                        table_name: Identifier::new("table_1".to_string()).unwrap(),
                        actions: vec![
                            AlterTableAction::AlterColumn(AlterColumn {
                                column_name: Identifier::new("bool".to_string()).unwrap(),
                                actions: vec![
                                    AlterColumnAction::SetType(DatabaseColumnType::String),
                                    AlterColumnAction::SetNullability(true),
                                ]
                            }),
                            AlterTableAction::AlterColumn(AlterColumn {
                                column_name: Identifier::new("int".to_string()).unwrap(),
                                actions: vec![AlterColumnAction::SetType(
                                    DatabaseColumnType::Float
                                ),]
                            }),
                            AlterTableAction::AddColumn(TableColumn {
                                parent_table_name: Identifier::new("table_1".to_string()).unwrap(),
                                column_name: Identifier::new("new_column".to_string()).unwrap(),
                                column_type: DatabaseColumnType::String,
                                is_primary_key: false,
                                is_nullable: false,
                            }),
                            AlterTableAction::AlterColumn(AlterColumn {
                                column_name: Identifier::new("string_nullable".to_string())
                                    .unwrap(),
                                actions: vec![AlterColumnAction::SetNullability(false),]
                            }),
                            AlterTableAction::DropColumn(
                                Identifier::new("timestamp".to_string()).unwrap()
                            ),
                        ],
                    }),
                    MigrationAction::DropTable(Identifier::new("table_2").unwrap()),
                    MigrationAction::CreateTable(CreateTable::new(table_3())),
                    MigrationAction::AlterTable(AlterTable {
                        table_name: Identifier::new("table_4".to_string()).unwrap(),
                        actions: vec![AlterTableAction::AddColumn(TableColumn {
                            parent_table_name: Identifier::new("table_4".to_string()).unwrap(),
                            column_name: Identifier::new("updated_at".to_string()).unwrap(),
                            column_type: DatabaseColumnType::Timestamp,
                            is_primary_key: false,
                            is_nullable: true,
                        }),],
                    }),
                ],
            }
        );

        println!("{}", migration.as_sql());
    }
}