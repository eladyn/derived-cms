use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use axum::extract::FromRef;

use crate::DB;

/// Trait implemented by the context available in all endpoints using [`axum::extract::State`].
pub trait ContextTrait: Clone + Send + Sync + 'static {
    type Ext: ContextExt<Self>;

    fn db(&self) -> &sqlx::Pool<DB>;
    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>>;
    fn uploads_dir(&self) -> &Path;
    fn ext(&self) -> &Self::Ext;
}

#[derive(Debug)]
pub struct Context<T: ContextExt<Self>> {
    pub(crate) names_plural: BTreeSet<&'static str>,
    pub(crate) db: sqlx::Pool<DB>,
    pub(crate) uploads_dir: PathBuf,
    pub(crate) ext: T,
}
impl<E: ContextExt<Self>> Clone for Context<E> {
    fn clone(&self) -> Self {
        Self {
            names_plural: self.names_plural.clone(),
            db: self.db.clone(),
            uploads_dir: self.uploads_dir.clone(),
            ext: self.ext.clone(),
        }
    }
}
impl<E: ContextExt<Self> + 'static> ContextTrait for Context<E> {
    type Ext = E;

    fn db(&self) -> &sqlx::Pool<DB> {
        &self.db
    }
    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>> {
        self.names_plural.iter()
    }
    fn uploads_dir(&self) -> &Path {
        &self.uploads_dir
    }
    fn ext(&self) -> &E {
        &self.ext
    }
}

impl FromRef<Context<()>> for () {
    fn from_ref(_input: &Context<()>) -> Self {}
}

pub trait ContextExt<Ctx>: FromRef<Ctx> + Clone + Send + Sync {}

impl<Ctx, T: Send + Sync + 'static> ContextExt<Ctx> for T where T: FromRef<Ctx> + Clone {}
