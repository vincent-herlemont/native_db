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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = server::server::build_http_server::<Item>("some.db".to_string()).await?;
    server.await?;
    Ok(())
}
