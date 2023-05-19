#![allow(dead_code, unused)]
use struct_db::{ReadableTable, SDBItem};

use crate::error::HttpErrResp;

pub type Result<T> = std::result::Result<T, HttpErrResp>;

#[async_trait::async_trait]
pub trait SDBApiContext<T>: Send + Sync + 'static
where
    T: SDBItem + Send + Sync + 'static,
{
    async fn get(&self, id: &[u8]) -> Result<Option<T>>;
    async fn insert(&mut self, item: T) -> Result<()>;
    async fn update(&self, item: T) -> Result<Option<&T>>;
    async fn delete(&mut self, id: &[u8]) -> Result<()>;
}

pub struct RawSDBApiContext {
    pub db: struct_db::Db,
}

#[async_trait::async_trait]
impl<T> SDBApiContext<T> for RawSDBApiContext
where
    T: SDBItem + Send + Sync + 'static,
{
    async fn get(&self, id: &[u8]) -> Result<Option<T>> {
        let tx = self.db.read_transaction().unwrap();
        let mut tables = tx.tables();
        let res: Option<T> = tables.primary_get(&tx, id).unwrap();
        Ok(res)
    }
    async fn insert(&mut self, item: T) -> Result<()> {
        let tx = self.db.transaction().unwrap();
        let s = &item;
        let mut tables = tx.tables().insert(&tx, item);
        tx.commit().unwrap();
        Ok(())
    }
    async fn update(&self, item: T) -> Result<Option<&T>> {
        todo!()
    }
    async fn delete(&mut self, id: &[u8]) -> Result<()> {
        todo!()
    }
}
