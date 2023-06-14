use std::sync::{Arc, RwLock};

use hashbrown::{hash_map::Entry, HashMap};
use thiserror::Error;

use crate::table::{Meta as TableMeta, Table};

#[derive(Error, Debug)]
pub enum DBError {
    #[error("table name {} has already existed", .name)]
    TableExists { name: String },
}

#[derive(Debug, Default)]
pub struct DB {
    tables: Vec<Arc<Table>>,
    index: HashMap<Arc<str>, usize>,
    tags: HashMap<Arc<str>, Vec<usize>>,
}

impl DB {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            ..Default::default()
        }))
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&Arc<Table>> {
        self.index.get(name).map(|id| &self.tables[*id])
    }

    pub fn create_table(&mut self, name: Arc<str>, meta: TableMeta) -> Result<(), DBError> {
        match self.index.entry(name.clone()) {
            Entry::Occupied(_) => {
                return Err(DBError::TableExists {
                    name: name.as_ref().to_owned(),
                })
            }
            Entry::Vacant(entry) => {
                self.tables.push(Arc::new(Table::new(name, meta)));
                entry.insert(self.tables.len() - 1);
                Ok(())
            }
        }
    }

    pub fn tag(&mut self, table: &str, tags: &[Arc<str>]) {
        if let Some(&table) = self.index.get(table) {
            for tag in tags {
                match self.tags.entry(tag.clone()) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(table);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![table]);
                    }
                }
            }
        }
    }
}

pub mod tests {
    use std::sync::{Arc, RwLock};

    use common::{
        column::{field::Field, label::LabelType},
        index::Index,
        schema::{self, Schema},
    };

    use super::DB;
    use crate::table::{ChunkMeta, Meta, MutableMeta};

    pub fn test_db() -> Arc<RwLock<DB>> {
        let db = DB::new();
        db.write()
            .unwrap()
            .create_table(
                Arc::from("foo.bar.something_used"),
                Meta {
                    chunk: ChunkMeta {
                        mutable: MutableMeta {
                            width: 1,
                            length: 1,
                            count: 1,
                        },
                    },
                    schema: Arc::new(Schema {
                        labels: vec![
                            schema::Label {
                                r#type: LabelType::String(()),
                                name: "env".into(),
                            },
                            schema::Label {
                                r#type: LabelType::String(()),
                                name: "status".into(),
                            },
                        ],
                        fields: vec![schema::Field {
                            r#type: Field::Float64(()).into(),
                            name: "value".into(),
                        }],
                        index: vec![Index::Inverted(())],
                    }),
                },
            )
            .unwrap();
        db
    }
}
