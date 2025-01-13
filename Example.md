# Example of axum service using utoipa for openapi

Below is an example of the code we want generated from the templates. 

Note that it generates a specific style of axum rest api that uses utoipa to add openapi documentation. e

```rust
use std::net::{Ipv4Addr, SocketAddr};

use std::io::Error;
use tokio::net::TcpListener;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

const TODO_TAG: &str = "todo";

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[derive(OpenApi)]
    #[openapi(
        modifiers(&SecurityAddon),
        tags(
            (name = TODO_TAG, description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
                )
            }
        }
    }

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1/todos", todo::router())
        .split_for_parts();

    let router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .merge(Redoc::with_url("/redoc", api.clone()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", api));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    axum::serve(listener, router.into_make_service()).await
}

mod todo {
    use std::sync::Arc;

    use axum::{
        extract::{Path, Query, State},
        response::IntoResponse,
        Json,
    };
    use hyper::{HeaderMap, StatusCode};
    use serde::{Deserialize, Serialize};
    use tokio::sync::Mutex;
    use utoipa::{IntoParams, ToSchema};
    use utoipa_axum::{router::OpenApiRouter, routes};

    use crate::TODO_TAG;

    /// In-memory todo store
    type Store = Mutex<Vec<Todo>>;

    /// Item to do.
    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    struct Todo {
        id: i32,
        #[schema(example = "Buy groceries")]
        value: String,
        done: bool,
    }

    /// Todo operation errors
    #[derive(Serialize, Deserialize, ToSchema)]
    enum TodoError {
        /// Todo already exists conflict.
        #[schema(example = "Todo already exists")]
        Conflict(String),
        /// Todo not found by id.
        #[schema(example = "id = 1")]
        NotFound(String),
        /// Todo operation unauthorized
        #[schema(example = "missing api key")]
        Unauthorized(String),
    }

    pub(super) fn router() -> OpenApiRouter {
        let store = Arc::new(Store::default());
        OpenApiRouter::new()
            .routes(routes!(list_todos, create_todo))
            .routes(routes!(search_todos))
            .routes(routes!(mark_done, delete_todo))
            .with_state(store)
    }

    /// List all Todo items
    ///
    /// List all Todo items from in-memory storage.
    #[utoipa::path(
        get,
        path = "",
        tag = TODO_TAG,
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        )
    )]
    async fn list_todos(State(store): State<Arc<Store>>) -> Json<Vec<Todo>> {
        let todos = store.lock().await.clone();

        Json(todos)
    }

    /// Todo search query
    #[derive(Deserialize, IntoParams)]
    struct TodoSearchQuery {
        /// Search by value. Search is incase sensitive.
        value: String,
        /// Search by `done` status.
        done: bool,
    }

    /// Search Todos by query params.
    ///
    /// Search `Todo`s by query params and return matching `Todo`s.
    #[utoipa::path(
        get,
        path = "/search",
        tag = TODO_TAG,
        params(
            TodoSearchQuery
        ),
        responses(
            (status = 200, description = "List matching todos by query", body = [Todo])
        )
    )]
    async fn search_todos(
        State(store): State<Arc<Store>>,
        query: Query<TodoSearchQuery>,
    ) -> Json<Vec<Todo>> {
        Json(
            store
                .lock()
                .await
                .iter()
                .filter(|todo| {
                    todo.value.to_lowercase() == query.value.to_lowercase()
                        && todo.done == query.done
                })
                .cloned()
                .collect(),
        )
    }

    /// Create new Todo
    ///
    /// Tries to create a new Todo item to in-memory storage or fails with 409 conflict if already exists.
    #[utoipa::path(
        post,
        path = "",
        tag = TODO_TAG,
        responses(
            (status = 201, description = "Todo item created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError)
        )
    )]
    async fn create_todo(
        State(store): State<Arc<Store>>,
        Json(todo): Json<Todo>,
    ) -> impl IntoResponse {
        let mut todos = store.lock().await;

        todos
            .iter_mut()
            .find(|existing_todo| existing_todo.id == todo.id)
            .map(|found| {
                (
                    StatusCode::CONFLICT,
                    Json(TodoError::Conflict(format!(
                        "todo already exists: {}",
                        found.id
                    ))),
                )
                    .into_response()
            })
            .unwrap_or_else(|| {
                todos.push(todo.clone());

                (StatusCode::CREATED, Json(todo)).into_response()
            })
    }

    /// Mark Todo item done by id
    ///
    /// Mark Todo item done by given id. Return only status 200 on success or 404 if Todo is not found.
    #[utoipa::path(
        put,
        path = "/{id}",
        tag = TODO_TAG,
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 404, description = "Todo not found")
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            (), // <-- make optional authentication
            ("api_key" = [])
        )
    )]
    async fn mark_done(
        Path(id): Path<i32>,
        State(store): State<Arc<Store>>,
        headers: HeaderMap,
    ) -> StatusCode {
        match check_api_key(false, headers) {
            Ok(_) => (),
            Err(_) => return StatusCode::UNAUTHORIZED,
        }

        let mut todos = store.lock().await;

        todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .map(|todo| {
                todo.done = true;
                StatusCode::OK
            })
            .unwrap_or(StatusCode::NOT_FOUND)
    }

    /// Delete Todo item by id
    ///
    /// Delete Todo item from in-memory storage by id. Returns either 200 success of 404 with TodoError if Todo is not found.
    #[utoipa::path(
        delete,
        path = "/{id}",
        tag = TODO_TAG,
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 401, description = "Unauthorized to delete Todo", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            ("api_key" = [])
        )
    )]
    async fn delete_todo(
        Path(id): Path<i32>,
        State(store): State<Arc<Store>>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        match check_api_key(true, headers) {
            Ok(_) => (),
            Err(error) => return error.into_response(),
        }

        let mut todos = store.lock().await;

        let len = todos.len();

        todos.retain(|todo| todo.id != id);

        if todos.len() != len {
            StatusCode::OK.into_response()
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(TodoError::NotFound(format!("id = {id}"))),
            )
                .into_response()
        }
    }

    // normally you should create a middleware for this but this is sufficient for sake of example.
    fn check_api_key(
        require_api_key: bool,
        headers: HeaderMap,
    ) -> Result<(), (StatusCode, Json<TodoError>)> {
        match headers.get("todo_apikey") {
            Some(header) if header != "utoipa-rocks" => Err((
                StatusCode::UNAUTHORIZED,
                Json(TodoError::Unauthorized(String::from("incorrect api key"))),
            )),
            None if require_api_key => Err((
                StatusCode::UNAUTHORIZED,
                Json(TodoError::Unauthorized(String::from("missing api key"))),
            )),
            _ => Ok(()),
        }
    }
}
```

The code that is used to process the templates, and therefore the variables available for use in the templates is from the following:

```rust
use serde::Serialize;
use serde_json::Value;
use tera::Context;
use tera::Tera;

use crate::{discovery::Discovered, error::Error, tools};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, process::Command};

use super::openapi::Openapi;
use inflector::Inflector;

#[derive(Debug)]
pub struct Templates {
    pub list: Vec<Template>,
}

#[derive(Debug)]
pub enum Template {
    Models(ModelsTemplate),
    Endpoints(EndpointsTemplate),
    Tags(TagsTemplate),
    Static(StaticTemplate),
    File(FileTemplate),
}

#[derive(Debug)]
pub struct EndpointsTemplate {
    relative: PathBuf,
    filename: Filename,
    content_type: String,
    condition: Option<Condition>,
    group_by: GroupBy,
}

#[derive(Debug)]
pub struct TagsTemplate {
    relative: PathBuf,
    filename: Filename,
    content_type: String,
    condition: Option<Condition>,
}

#[derive(Debug)]
pub struct ModelsTemplate {
    relative: PathBuf,
    filename: Filename,
    condition: Option<Condition>,
}

#[derive(Debug)]
pub struct StaticTemplate {
    relative: PathBuf,
    filename: Filename,
    condition: Option<Condition>,
}

#[derive(Debug)]
pub struct Condition {
    pub kv: String,
}

#[derive(Debug, Default)]
pub struct GroupBy {
    pub kind: Option<String>,
}
#[derive(Serialize)]
pub struct TagContainer {
    tag: String,
    endpoints: Vec<super::openapi::endpoint::Endpoint>,
}

pub trait Group {
    fn process(&self, openapi: &mut Openapi, container: &mut super::CodegenContainer);
}

#[derive(Debug)]
pub struct FileTemplate {
    relative: String,
    path: PathBuf,
}

#[derive(Eq, PartialEq, Debug)]
pub enum TemplateType {
    Models,
    Endpoints,
}

#[derive(Debug, Clone)]
pub struct Filename {
    filename: String,
}

impl Filename {
    pub fn from(filename: String) -> Self {
        Self { filename }
    }

    pub fn resolve(&self, container: &super::CodegenContainer) -> Result<String, Error> {
        tools::fill_parameters(&self.filename, container)
    }
}

impl Condition {
    pub fn from(kv: &str) -> Result<Self, Error> {
        Ok(Self { kv: kv.to_string() })
    }

    pub fn check(&self, container: &super::CodegenContainer) -> bool {
        tools::fill_parameters(&self.kv, container)
            .map(|s| {
                let parts = s.split(':').collect::<Vec<&str>>();
                if let [left, right] = parts[..] {
                    left == right
                } else {
                    true
                }
            })
            .unwrap_or(false)
    }
}

impl GroupBy {
    pub fn from(group_by: &str) -> Result<Self, Error> {
        if group_by != "tag" {
            Err(Error::CodegenNotAllowedGroupBy(group_by.to_string()))
        } else {
            Ok(Self {
                kind: Some(group_by.to_string()),
            })
        }
    }

    pub fn split(&self, openapi: &Openapi) -> impl IntoIterator<Item = impl Group> {
        match &self.kind {
            Some(_) => TagGroup::produce(openapi)
                .into_iter()
                .map(GroupType::TagGroup)
                .collect::<Vec<_>>(),
            None => vec![GroupType::NoGroup],
        }
    }
}
pub struct TagGroup {
    tag: String,
}

impl Group for TagGroup {
    fn process(&self, openapi: &mut Openapi, container: &mut super::CodegenContainer) {
        container.data.insert(
            "tag".to_string(),
            Value::String(self.tag.clone().to_pascal_case()),
        );

        openapi
            .endpoints
            .retain(|s| s.get_tags().contains(&self.tag));
    }
}

impl TagGroup {
    pub fn produce(openapi: &Openapi) -> Vec<TagGroup> {
        let mut tags = openapi.endpoints.iter().fold(vec![], |mut acc, item| {
            acc.append(&mut item.get_tags().clone());
            acc
        });

        tags.sort();
        tags.dedup();

        tags.iter()
            .map(|t| TagGroup { tag: t.clone() })
            .collect::<Vec<_>>()
    }

    pub fn filter(
        &self,
        endpoints: &[super::openapi::endpoint::Endpoint],
    ) -> Vec<super::openapi::endpoint::Endpoint> {
        endpoints
            .iter()
            .filter(|e| e.get_tags().contains(&self.tag))
            .cloned()
            .collect()
    }
}

pub enum GroupType {
    TagGroup(TagGroup),
    NoGroup,
}

impl Group for GroupType {
    fn process(&self, openapi: &mut Openapi, container: &mut super::CodegenContainer) {
        match &self {
            Self::TagGroup(t) => t.process(openapi, container),
            Self::NoGroup => {}
        }
    }
}

impl Templates {
    pub fn includes(&self, types: &[TemplateType]) -> bool {
        self.list
            .iter()
            .filter_map(|t| match *t {
                Template::Models(_) => Some(TemplateType::Models),
                Template::Endpoints(_) => Some(TemplateType::Endpoints),
                _ => None,
            })
            .filter(|f| types.contains(f))
            .count()
            > 0
    }
}

impl Template {
    fn from_file(relative: String, path: PathBuf) -> Self {
        Template::File(FileTemplate { relative, path })
    }

    fn from_content(relative: String, content: String) -> Result<Self, Error> {
        let first = content.lines().next();

        if let Some(line) = first {
            let mut first_line = line.to_string();

            let last_hash = first_line
                .char_indices()
                .find(|&(_, c)| c != '#')
                .map_or(0, |(idx, _)| idx);
            first_line = first_line[last_hash..].trim().to_string();

            if !first_line.starts_with("{# ") {
                return Err(Error::CodegenFileSkipped);
            }

            let params = super::format(first_line.trim_matches(&['{', '}', '#', ' '] as &[_]))?;

            if let Some(serde_json::Value::String(min_version)) = params.get("min_version") {
                let min = semver::Version::parse(min_version).map_err(Error::SemVersion)?;
                let current = semver::Version::parse(crate::VERSION).unwrap();

                let req = semver::VersionReq::parse(
                    format!(">={}, <{}.{}.0", min_version, min.major + 1, min.minor).as_str(),
                )
                .unwrap();
                if !req.matches(&current) {
                    return Err(Error::IncorrectVersionError(current, min, relative));
                }
            } else {
                return Err(Error::MissingMinVersionError(relative));
            }

            params
                .get("type")
                .ok_or_else(|| Error::CodegenFileHeaderRequired("type".to_string()))?
                .as_str()
                .map(|type_| match type_ {
                    "endpoints" => EndpointsTemplate::from(PathBuf::from(relative), &params),
                    "models" => ModelsTemplate::from(PathBuf::from(relative), &params),
                    "tags" => TagsTemplate::from(PathBuf::from(relative), &params),
                    "static" => StaticTemplate::from(PathBuf::from(relative), &params),
                    _ => Err(Error::CodegenFileHeaderRequired("type".to_string())),
                })
                .unwrap()
        } else {
            Err(Error::CodegenFileSkipped)
        }
    }

    pub fn format(&self, command: &str, files: Vec<String>) -> Result<(), Error> {
        let parts = crate::tools::ArgumentsExtractor::new(command).collect::<Vec<String>>();

        for file in files {
            let mut cmd = Command::new(parts.first().unwrap());
            for i in 1..parts.len() {
                cmd.arg(parts.get(i).unwrap());
            }

            let output = cmd
                .arg(file)
                .output()
                .map_err(Error::CodegenFormattingError)?;

            if !output.status.success() {
                return Err(Error::CodegenFormattingCommandError(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl StaticTemplate {
    pub fn from(relative: PathBuf, config: &HashMap<&str, Value>) -> Result<Template, Error> {
        let filename = Filename::from(
            config
                .get("filename")
                .ok_or_else(|| Error::CodegenFileHeaderRequired("filename".to_string()))?
                .as_str()
                .unwrap()
                .to_string(),
        );

        let condition = config
            .get("if")
            .map(|s| Condition::from(s.as_str().unwrap()))
            .map_or(Ok(None), |v| v.map(Some))?;

        Ok(Template::Static(Self {
            relative,
            filename,
            condition,
        }))
    }

    pub fn render(
        &self,
        tera: &Tera,
        target_dir: &str,
        container: &super::CodegenContainer,
    ) -> Result<Vec<String>, Error> {
        if self
            .condition
            .as_ref()
            .map(|s| s.check(container))
            .unwrap_or(true)
        {
            process_render(
                tera,
                serde_json::json!({}),
                PathBuf::from(format!(
                    "{}/{}",
                    target_dir,
                    self.filename.resolve(container)?
                )),
                self.relative.clone(),
                container,
            )
        } else {
            log::info!("Template skipped due to condition: {:?}", self.relative);

            Ok(vec![])
        }
    }
}

impl EndpointsTemplate {
    pub fn from(relative: PathBuf, config: &HashMap<&str, Value>) -> Result<Template, Error> {
        let filename = Filename::from(
            config
                .get("filename")
                .ok_or_else(|| Error::CodegenFileHeaderRequired("filename".to_string()))?
                .as_str()
                .unwrap()
                .to_string(),
        );

        let content_type = config
            .get("content_type")
            .map(|s| s.as_str().unwrap().to_string())
            .unwrap_or_else(|| "application/json".to_string());

        let condition = config
            .get("if")
            .map(|s| Condition::from(s.as_str().unwrap()))
            .map_or(Ok(None), |v| v.map(Some))?;

        let group_by = config
            .get("group_by")
            .map(|s| GroupBy::from(s.as_str().unwrap()))
            .unwrap_or_else(|| Ok(GroupBy::default()))?;

        Ok(Template::Endpoints(Self {
            relative,
            filename,
            content_type,
            condition,
            group_by,
        }))
    }

    pub fn render(
        &self,
        tera: &Tera,
        target_dir: &str,
        openapi: &super::openapi::Openapi,
        container: &super::CodegenContainer,
    ) -> Result<Vec<String>, Error> {
        let mut result = vec![];

        for group in self.group_by.split(openapi) {
            // prepare per group structures
            let mut openapi = openapi.clone().set_content_type(&self.content_type);
            let mut container = container.clone();

            container.data.insert(
                "formats".to_string(),
                serde_json::to_value(openapi.models.formats()).unwrap(),
            );

            // process group
            group.process(&mut openapi, &mut container);

            if self
                .condition
                .as_ref()
                .map(|s| s.check(&container))
                .unwrap_or(true)
            {
                // render
                result.append(&mut process_render(
                    tera,
                    openapi,
                    PathBuf::from(format!(
                        "{}/{}",
                        target_dir,
                        self.filename.resolve(&container)?
                    )),
                    self.relative.clone(),
                    &container,
                )?)
            } else {
                log::info!("Template skipped due to condition: {:?}", self.relative);
            }
        }

        Ok(result)
    }
}

impl TagsTemplate {
    pub fn from(relative: PathBuf, config: &HashMap<&str, Value>) -> Result<Template, Error> {
        let filename = Filename::from(
            config
                .get("filename")
                .ok_or_else(|| Error::CodegenFileHeaderRequired("filename".to_string()))?
                .as_str()
                .unwrap()
                .to_string(),
        );

        let content_type = config
            .get("content_type")
            .map(|s| s.as_str().unwrap().to_string())
            .unwrap_or_else(|| "application/json".to_string());

        let condition = config
            .get("if")
            .map(|s| Condition::from(s.as_str().unwrap()))
            .map_or(Ok(None), |v| v.map(Some))?;

        Ok(Template::Tags(Self {
            relative,
            filename,
            content_type,
            condition,
        }))
    }

    pub fn render(
        &self,
        tera: &Tera,
        target_dir: &str,
        openapi: &super::openapi::Openapi,
        container: &super::CodegenContainer,
    ) -> Result<Vec<String>, Error> {
        let groups = TagGroup::produce(openapi);

        let mut tags: Vec<TagContainer> = vec![];
        let mut processed = openapi.clone().set_content_type(&self.content_type);
        let mut container = container.clone();

        for group in groups {
            tags.push(TagContainer {
                tag: group.tag.clone().to_pascal_case(),
                endpoints: group.filter(&openapi.endpoints),
            })
        }

        processed.endpoints = vec![];

        container
            .data
            .insert("tags".to_string(), serde_json::to_value(tags).unwrap());

        container.data.insert(
            "formats".to_string(),
            serde_json::to_value(openapi.models.formats()).unwrap(),
        );

        if self
            .condition
            .as_ref()
            .map(|s| s.check(&container))
            .unwrap_or(true)
        {
            // render
            process_render(
                tera,
                processed,
                PathBuf::from(format!(
                    "{}/{}",
                    target_dir,
                    self.filename.resolve(&container)?
                )),
                self.relative.clone(),
                &container,
            )
        } else {
            log::info!("Template skipped due to condition: {:?}", self.relative);
            Ok(vec![])
        }
    }
}

impl ModelsTemplate {
    pub fn from(relative: PathBuf, config: &HashMap<&str, Value>) -> Result<Template, Error> {
        let filename = Filename::from(
            config
                .get("filename")
                .ok_or_else(|| Error::CodegenFileHeaderRequired("filename".to_string()))?
                .as_str()
                .unwrap()
                .to_string(),
        );

        let condition = config
            .get("if")
            .map(|s| Condition::from(s.as_str().unwrap()))
            .map_or(Ok(None), |v| v.map(Some))?;

        Ok(Template::Models(Self {
            relative,
            filename,
            condition,
        }))
    }

    pub fn render(
        &self,
        tera: &Tera,
        target_dir: &str,
        models: &super::jsonschema::ModelContainer,
        container: &super::CodegenContainer,
    ) -> Result<Vec<String>, Error> {
        if self
            .condition
            .as_ref()
            .map(|s| s.check(container))
            .unwrap_or(true)
        {
            process_render(
                tera,
                models,
                PathBuf::from(format!(
                    "{}/{}",
                    target_dir,
                    self.filename.resolve(container)?
                )),
                self.relative.clone(),
                container,
            )
        } else {
            log::info!("Template skipped due to condition: {:?}", self.relative);

            Ok(vec![])
        }
    }
}

impl FileTemplate {
    pub fn copy(&self, target_dir: &str) -> Result<Vec<String>, Error> {
        let target = PathBuf::from(format!("{}/{}", target_dir, self.relative));

        log::info!("Copying: {:?}", target);

        let mut directory = target.clone();
        directory.pop();

        std::fs::create_dir_all(directory).map_err(|e| Error::CodegenFileError(e.to_string()))?;

        std::fs::copy(self.path.clone(), target.clone())
            .map(|_| ())
            .map_err(|e| Error::CodegenFileError(e.to_string()))?;

        Ok(vec![target.to_string_lossy().to_string()])
    }
}

pub fn get(discovered: Discovered) -> Result<Templates, Error> {
    let mut list: Vec<Template> = vec![];

    for (relative, content) in discovered.templates {
        let result = Template::from_content(relative.clone(), content);

        match result {
            Ok(template) => {
                list.push(template);
            }
            Err(err) => match err {
                Error::CodegenFileSkipped => {
                    log::trace!("file skipped: {}", relative);
                    continue;
                }
                e => return Err(e),
            },
        }
    }

    for (relative, path) in discovered.files {
        list.push(Template::from_file(relative, path))
    }

    if list.is_empty() {
        return Err(Error::CodegenTemplatesDirectoryError);
    }

    Ok(Templates { list })
}

fn process_render(
    tera: &Tera,
    data: (impl Serialize + Clone),
    target: PathBuf,
    relative: PathBuf,
    container: &super::CodegenContainer,
) -> Result<Vec<String>, Error> {
    let mut ctx = Context::from_serialize(serde_json::to_value(data).unwrap()).unwrap();

    let data = serde_json::to_value(container).unwrap();
    for (key, value) in data.as_object().unwrap() {
        ctx.insert(key, value);
    }

    let result = tera
        .render(&relative.to_string_lossy(), &ctx)
        .map_err(Error::CodegenTemplateError)?;

    if result.trim().is_empty() {
        return Ok(vec![]);
    }

    log::info!("Rendering: {:?}", target);

    let mut directory = target.clone();
    directory.pop();

    std::fs::create_dir_all(directory).map_err(|e| Error::CodegenFileError(e.to_string()))?;

    let mut file =
        File::create(target.clone()).map_err(|e| Error::CodegenFileError(e.to_string()))?;

    file.write_all(result.as_bytes())
        .map_err(|e| Error::CodegenFileError(e.to_string()))?;

    Ok(vec![target.to_string_lossy().to_string()])
}
```