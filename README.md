# derived-cms

Generate a CMS, complete with admin interface and headless API interface from Rust type definitions.
Works in cunjunction with [serde](https://docs.rs/serde/latest/serde/) and
[ormlite](https://lib.rs/crates/ormlite) and uses [axum](https://docs.rs/axum/latest/axum/)
as a web server.

Example

```rust
use chrono::{DateTime, Utc};
use derived_cms::{App, Entity, EntityBase, Input, app::AppError, context::{Context, ContextTrait}, entity, property::{Markdown, Text, Json}};
use ormlite::{Model, sqlite::Sqlite};
use serde::{Deserialize, Serialize, Serializer};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Entity, Model, TS)]
#[ts(export)]
struct Post {
    #[cms(id, skip_input)]
    #[ormlite(primary_key)]
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    title: Text,
    date: DateTime<Utc>,
    #[cms(skip_column)]
    #[serde(default)]
    content: Json<Vec<Block>>,
    draft: bool,
}

type Ctx = Context<ormlite::Pool<sqlx::Sqlite>>;

impl entity::Get<Ctx> for Post {
    type RequestExt = State<Ctx>;
    type Error = MyError;

    async fn get(
        id: &<Self as EntityBase<Ctx>>::Id,
        ext: Self::RequestExt,
    ) -> Result<Option<Self>, Self::Error> {
        match Self::fetch_one(id, ext.ext()).await {
            Ok(v) => Ok(Some(v)),
            Err(ormlite::Error::SqlxError(sqlx::Error::RowNotFound)) => Ok(None),
            Err(e) => Err(e)?,
        }
    }
}

impl entity::List<Ctx> for Post {
    type RequestExt = State<Ctx>;
    type Error = MyError;

    async fn list(ext: Self::RequestExt) -> Result<impl IntoIterator<Item = Self>, Self::Error> {
        Ok(Self::select().fetch_all(ext.ext()).await?)
    }
}

impl entity::Create<Ctx> for Post {
    type RequestExt = State<Ctx>;
    type Error = MyError;

    async fn create(
        data: <Self as EntityBase<Ctx>>::Create,
        ext: Self::RequestExt,
    ) -> Result<Self, Self::Error> {
        Ok(Self::insert(data, ext.ext()).await?)
    }
}

impl entity::Update<Ctx> for Post {
    type RequestExt = State<Ctx>;
    type Error = MyError;

    async fn update(
        id: &<Self as EntityBase<Ctx>>::Id,
        mut data: <Self as EntityBase<Ctx>>::Update,
        ext: Self::RequestExt,
    ) -> Result<Self, Self::Error> {
        Ok(data.update_all_fields(ext.ext()).await?)
    }
}

impl entity::Delete<Ctx> for Post {
    type RequestExt = State<Ctx>;
    type Error = MyError;

    async fn delete(
        id: &<Self as EntityBase<Ctx>>::Id,
        ext: Self::RequestExt,
    ) -> Result<(), Self::Error> {
        let r = sqlx::query("DELETE FROM post WHERE id = ?")
            .bind(id)
            .execute(ext.ext())
            .await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Input, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Block {
    Separator,
    Text(Markdown),
}

#[tokio::main]
async fn main() {
    let db = sqlx::Pool::<Sqlite>::connect("sqlite://.tmp/db.sqlite")
        .await
        .unwrap();
    let app = App::new().entity::<Post>().with_state(db).build("uploads");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## REST API

A REST API is automatically generated for all `Entities`.

List of generated endpoints, with `name` and `name-plural`
converted to kebab-case:

- `GET /api/v1/:name-plural`:
  - allows filtering by exact value in the query string, e. g. `?slug=asdf`. This currently
    only works for fields whose SQL representation is a string.
  - returns an array of entities, serialized using [serde_json](https://docs.rs/serde-json/latest/serde_json).
- `GET /api/v1/:name/:id`
  - get an Entity by it's id.
  - returns the requested of Entity, serialized using [serde_json](https://docs.rs/serde-json/latest/serde_json).
- `POST /api/v1/:name-plural`
  - create a new Entity from the request body JSON.
  - returns the newly created Entity as JSON.
- `POST /api/v1/:name/:id`
  - replaces the Entity with the specified id with the
    request body JSON.
  - returns the updated Entity as JSON.
- `DELETE /api/v1/:name/:id`
  - deletes the Entity with the specified id
  - returns the deleted Entity as JSON.
