use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[struct_db::struct_db(fn_primary_key(p_key))]
struct Item {
    id: i32,
    name: String,
}

impl Item {
    pub fn p_key(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }
}

// insert some data data in some.db as a test since only get request is implemented so far
fn insert_some_data() -> anyhow::Result<()> {
    let path = std::path::Path::new("some.db");
    let mut db = struct_db::Db::init(&path).unwrap();
    db.define::<Item>();
    let item = Item {
        id: 1,
        name: "some name".to_string(),
    };
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        let res = tables.insert(&tx, item);
        println!("insert {:?}", res);
    }
    tx.commit().unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let r = insert_some_data();
    println!("{:?}", r);
    // use the same db as before
    let server = server::server::build_http_server::<Item>("some.db".to_string()).await?;
    // make request to http://localhost:8080/item/1 to response
    server.await?;
    Ok(())
}
