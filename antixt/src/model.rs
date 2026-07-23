use crate::Method;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteParam {
    pub name: String,
    pub catch_all: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteSource {
    pub method: Method,
    pub path: String,
    pub source: PathBuf,
    pub layouts: Vec<PathBuf>,
    pub function: &'static str,
    pub params: Vec<RouteParam>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientSource {
    pub name: String,
    pub source: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub directory: PathBuf,
    pub config: Option<PathBuf>,
    pub components: Option<PathBuf>,
    pub routes: Vec<RouteSource>,
    pub clients: Vec<ClientSource>,
}
