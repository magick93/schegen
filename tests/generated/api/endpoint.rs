
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
        (status = 201, description = "API key generated", body = ApiKeyResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    operation_id = "generate_api_key",
    tag = "auth"
)]
pub async fn generate_api_key(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let api_key = uuid::Uuid::new_v4().to_string();
    let mut keys = state.api_keys.write().await;
    keys.push(api_key.clone());
    
    Ok(Json(ApiKeyResponse {
        key: api_key
    }))
}

#[utoipa::path(
    get,
    path = "/auth/rate-limit",
    responses(
        (status = 200, description = "Rate limit status", body = RateLimitResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    operation_id = "rate_limit_status",
    tag = "auth"
)]
pub async fn rate_limit_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Ok(Json(RateLimitResponse {
        remaining: state.rate_limit,
        reset: 60
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    let api_routes = Router::new()
        .route("/", get(hello_world))
        .route("/items", get(list_items).post(create_item))
        .route("/items/:id", get(get_item))
        .route("/api-docs", get(|| async {
            Json(openapi())
        }))
        .route("/openapi.json", get(|| async {
            Json(openapi())
        }));
        
    let auth_routes = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/api-key", post(generate_api_key))
        .route("/rate-limit", get(rate_limit_status));
        
    Router::new()
        .nest("/api", api_routes)
        .nest("/auth", auth_routes)
        .merge(openapi().into_router("/api-docs"))
        .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
        .with_state(Arc::new(AppState {
            jwt_secret: "secret".to_string(),
            api_keys: Arc::new(RwLock::new(Vec::new())),
            rate_limit: 100,
        }))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        hello_world,
        create_item,
        list_items,
        get_item,
        login,
        logout,
        generate_api_key,
        rate_limit_status
    ),
    components(schemas(
        HelloWorldResponse,
        ErrorResponse,
        NotFoundResponse,
        ValidationErrorResponse,
        CreateRequest,
        ItemResponse,
        ListResponse<ItemResponse>,
        AuthResponse,
        ApiKeyResponse,
        RateLimitResponse,
        LoginRequest,
        Claims
    )),
    tags(
        (name = "example", description = "Example API endpoints"),
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.info.title = "Example API".to_string();
    openapi.info.description = Some("Example API documentation".to_string());
    openapi.info.version = "1.0.0".to_string();
    openapi.servers = Some(vec![
        utoipa::openapi::Server::new("/api")
            .description("API server"),
        utoipa::openapi::Server::new("/auth")
            .description("Authentication server")
    ]);
    openapi
}
