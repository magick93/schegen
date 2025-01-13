
use axum::{
    response::IntoResponse,
    Json,
    http::StatusCode,
    routing::{get, post, put, delete},
    Router,
    extract::{Path, Query, Json as ExtractJson, State, FromRequestParts},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use axum_extra::extract::CookieJar;
use utoipa::{ToSchema, ToResponse, OpenApi};
use serde::{Serialize, Deserialize};
use validator::Validate;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, Header, Algorithm};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::limit::RateLimitLayer;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub api_keys: Arc<RwLock<Vec<String>>>,
    pub rate_limit: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[response(description = "Standard success response")]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

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

#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[response(description = "Authentication response containing JWT token")]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[response(description = "API key generation response")]
pub struct ApiKeyResponse {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[response(description = "Rate limit status response")]
pub struct RateLimitResponse {
    pub remaining: u32,
    pub reset: u64,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, IntoResponse, ToResponse)]
#[response(description = "Standard success response containing a message")]
pub struct HelloWorldResponse {
    /// The message returned by the API
    message: String,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, IntoResponse, ToResponse)]
#[response(description = "Standard error response containing error details")]
pub struct ErrorResponse {
    /// Description of the error
    error: String,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, IntoResponse, ToResponse)]
#[response(description = "Resource not found response with explanation")]
pub struct NotFoundResponse {
    /// Description of the not found error
    message: String,
}

#[derive(Debug, ToSchema, Serialize, IntoResponse, ToResponse)]
#[response(description = "Validation error response with list of validation issues")]
pub struct ValidationErrorResponse {
    /// List of validation errors
    errors: Vec<String>,
}

/// In-memory todo store
type Store = Mutex<Vec<Todo>>;

mod todo {
    use super::*;

    pub(super) fn router() -> OpenApiRouter {
        let store = Arc::new(Store::default());
        OpenApiRouter::new()
            .routes(routes!(list_todos))
            .with_state(store)
    }

    /// List all Todo items
    ///
    /// List all Todo items from in-memory storage.
    #[utoipa::path(
        get,
        path = "",
        tag = "todo",
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        )
    )]
    pub async fn list_todos(State(store): State<Arc<Store>>) -> Json<Vec<Todo>> {
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
        tag = "todo",
        params(
            TodoSearchQuery
        ),
        responses(
            (status = 200, description = "List matching todos by query", body = [Todo])
        )
    )]
    pub async fn search_todos(
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
        tag = "todo",
        responses(
            (status = 201, description = "Todo item created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError)
        )
    )]
    pub async fn create_todo(
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
        tag = "todo",
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
    pub async fn mark_done(
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

    /// Delete Todo item by id
    ///
    /// Delete Todo item from in-memory storage by id. Returns either 200 success of 404 with TodoError if Todo is not found.
    #[utoipa::path(
        delete,
        path = "/{id}",
        tag = "todo",
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
    pub async fn delete_todo(
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
}

pub(super) fn router() -> OpenApiRouter {
    let store = Arc::new(Store::default());
    OpenApiRouter::new()
        .routes(routes!(list_todos, create_todo, mark_done, delete_todo))
        .with_state(store)
}

#[derive(Debug, ToSchema, Serialize, IntoResponse, ToResponse)]
#[response(description = "Standard success response with status and message")]
pub struct SuccessResponse {
    /// Indicates if the operation was successful
    success: bool,
    /// Additional success message
    message: String,
}

#[derive(Debug, ToSchema, Serialize, IntoResponse, ToResponse)]
#[response(description = "Single item response with item details")]
pub struct ItemResponse {
    /// Unique identifier of the item
    id: i64,
    /// Name of the item
    name: String,
    /// Creation timestamp of the item
    created_at: String,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, IntoResponse, ToResponse)]
#[response(description = "Paginated list response with items and total count")]
#[schema(example = json!({
    "items": [{
        "id": 1,
        "name": "Example Item",
        "created_at": "2024-01-01T00:00:00Z"
    }],
    "total": 1
}))]
pub struct ListResponse<T>
where
    T: ToResponse + ToSchema + Serialize,
{
    /// List of items
    items: Vec<T>,
    /// Total number of items available
    total: i64,
}

#[derive(ToSchema, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(ToSchema, Deserialize, Validate)]
pub struct CreateRequest {
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(range(min = 1, max = 100))]
    pub quantity: i32,
}

/// Returns a simple hello world message
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Returns a hello world message", body = HelloWorldResponse),
        (status = 400, description = "Invalid request parameters", body = ValidationErrorResponse),
        (status = 404, description = "Requested resource not found", body = NotFoundResponse),
        (status = 500, description = "Internal server error occurred", body = ErrorResponse)
    ),
    operation_id = "get_hello_world",
    tag = "example",
    security(
        ("api_key" = [])
    )
)]
pub async fn hello_world() -> impl IntoResponse {
    Json(HelloWorldResponse { message: "Hello World".to_string() })
}

/// Get an item by its ID
#[utoipa::path(
    get,
    path = "/items/{id}",
    responses(
        (status = 200, description = "Returns the requested item", body = ItemResponse),
        (status = 404, description = "Item with specified ID not found", body = NotFoundResponse),
        (status = 500, description = "Internal server error occurred", body = ErrorResponse)
    ),
    operation_id = "get_item_by_id",
    tag = "example"
)]
pub async fn get_item(Path(id): Path<i64>) -> impl IntoResponse {
    Json(ItemResponse {
        id,
        name: "Example Item".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string()
    })
}

/// List all items
#[utoipa::path(
    get,
    path = "/items",
    responses(
        (status = 200, description = "Returns paginated list of items", body = ListResponse<ItemResponse>),
        (status = 500, description = "Internal server error occurred", body = ErrorResponse)
    ),
    operation_id = "list_all_items",
    tag = "example"
)]
pub async fn list_items() -> impl IntoResponse {
    Json(ListResponse {
        items: vec![ItemResponse {
            id: 1,
            name: "Example Item".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string()
        }],
        total: 1
    })
}

/// Create a new item







#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Success", body = String, content_type = "application/json"),
        (status = 400, description = "Bad Request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not Found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    params(
        ),
    request_body = ()
)]
pub async fn create_item(ExtractJson(payload): ExtractJson<CreateRequest>) -> Result<impl IntoResponse, Json<ValidationErrorResponse>> {
    if let Err(errors) = payload.validate() {
        let error_messages = errors.field_errors()
            .into_iter()
            .map(|(_, errors)| errors[0].message.clone().unwrap_or_default())
            .collect();
            
        return Err(Json(ValidationErrorResponse { errors: error_messages }));
    }

    Ok(Json(SuccessResponse { success: true, message: "Item created".to_string() }))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    operation_id = "login",
    tag = "auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    ExtractJson(payload): ExtractJson<LoginRequest>
) -> Result<Json<AuthResponse>, Json<ErrorResponse>> {
    let claims = Claims {
        sub: "user_id".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        role: "user".to_string()
    };
    
    match generate_jwt(&claims, &state.jwt_secret) {
        Ok(token) => Ok(Json(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: 3600
        })),
        Err(_) => Err(Json(ErrorResponse {
            error: "Failed to generate token".to_string()
        }))
    }
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    operation_id = "logout",
    tag = "auth"
)]
pub async fn logout() -> impl IntoResponse {
    Ok(Json(SuccessResponse {
        success: true,
        message: "Logged out successfully".to_string()
    }))
}

#[utoipa::path(
    post,
    path = "/auth/api-key",
    responses(
        (status
