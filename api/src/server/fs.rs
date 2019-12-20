use crate::{error::ServerError, server::Pool, vfs::VirtualFileSystem};
use actix_web::{get, web, HttpResponse};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    Exists,
    Metadata,
    Content,
    List,
}

#[derive(Debug, Deserialize)]
pub struct VfsQuery {
    operation: FileOperation,
}

#[derive(Debug, Serialize)]
struct Resp<T: Serialize> {
    content: T,
}

impl<T: Serialize> Resp<T> {
    fn serialize(content: T) -> Result<String, serde_json::Error> {
        let resp = Resp { content };
        serde_json::to_string(&resp)
    }
}

/// Request handler to serve and GET request for the virtual file system.
#[get("/files{path:($|/.*$)}")] // path can be empty or start with a slash
pub fn file_system_get(
    pool: web::Data<Pool>,
    path: web::Path<String>,
    web::Query(query): web::Query<VfsQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let db_conn = &pool.get().map_err(ServerError::from)? as &PgConnection;
    let fs = VirtualFileSystem::new(db_conn);

    let response_content: String = match query.operation {
        FileOperation::Exists => Resp::serialize(fs.exists(&path)?)?,
        FileOperation::Metadata => Resp::serialize(&fs.get_metadata(&path)?)?,
        FileOperation::Content => Resp::serialize(fs.get_content(&path)?)?,
        FileOperation::List => Resp::serialize(fs.list_children(&path)?)?,
    };

    Ok(response_content.into())
}
