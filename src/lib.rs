//! Represents the initialization of a trip with details such as destination,
//! duration, and additional response information.
//!
//! # Fields
//! * `destination` (`String`) - The chosen trip destination.
//! * `days` (`u32`) - Duration of the trip in days.
//! * `response` (`String`) - A status or response message specific to the trip's initialization.
//!
//! # Notes
//! - This struct is serializable and deserializable to formats such as JSON through the use
//!   of the `serde` crate.
//! - It is created as part of the process to set up and manage trip data.
use uuid::Uuid;
use worker::*;
use serde::{Serialize, Deserialize};
mod db;
mod ai;

use db::create_trip;
use crate::db::{check_if_messages, create_message, get_messages};

/// The `TripInit` struct represents the initialization details of a trip,
/// including the destination, duration, and a response message.
///
/// # Attributes
///
/// * `destination` (`String`): The destination of the trip.
/// * `days` (`u32`): The number of days the trip will last.
/// * `response` (`String`): A response or status message related to the trip initialization.
///
/// This struct derives the `Serialize` and `Deserialize` traits to allow easy
/// conversion to and from formats such as JSON or other serialized data representations.
#[derive(Serialize, Deserialize)]
struct TripInit {
    destination: String,
    days: u32,
    response: String,
}


/// A data structure representing information about a trip.
///
/// # Fields
///
/// * `id` - A unique identifier for the trip, represented as a `String`.
/// * `destination` - The destination of the trip, represented as a `String`.
/// * `days` - The number of days the trip will last, represented as a `u32`.
///
/// This struct derives the following traits:
/// * `Serialize` - Enables the struct to be serialized into formats such as JSON.
/// * `Deserialize` - Enables the struct to be deserialized from formats such as JSON.
/// * `Clone` - Allows the struct to be cloned, creating a duplicate instance.
///
/// # Example
///
/// ```
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone)]
/// pub struct TripData {
///     pub id: String,
///     pub destination: String,
///     pub days: u32,
/// }
///
/// let trip = TripData {
///     id: String::from("trip123"),
///     destination: String::from("Hawaii"),
///     days: 7,
/// };
/// println!("Trip to {} for {} days", trip.destination, trip.days);
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct TripData {
   pub id: String,
   pub destination: String,
   pub days: u32,
}

/// The `main` function serves as the entry point for handling incoming HTTP requests.
/// It routes requests to appropriate handlers based on HTTP method, URL path, and headers.
///
/// # Parameters
/// - `req`: The incoming `Request` object containing information like method, path, headers, and body.
/// - `env`: The `Env` object representing the runtime environment/context of the application.
/// - `_ctx`: The `Context` object, currently unused, but available for additional context.
///
/// # Returns
/// - Returns a `Result<Response>` where `Response` is the HTTP response sent back to the client.
/// - In case of an error during processing, a `Response::error` with status code `404` or another appropriate error response is returned.
///
/// # Routing Logic
/// 1. **GET `/`:**
///    Calls the `index` handler to serve the root endpoint.
///
/// 2. **POST `/input`:**
///    Calls the `input` handler with the request, environment, and context to process the input endpoint.
///
/// 3. **GET `/trip/{trip_id}`:**
///    - Extracts the `trip_id` from the URL path.
///    - Checks the `Accept` header:
///        - If it contains `text/html`, serves an HTML page (`chat.html`).
///        - Otherwise, processes the request by calling the `get_trip` handler to fetch trip details.
///
/// 4. **POST `/trip/{trip_id}`:**
///    Calls the `chat` handler with the request, environment, and context to process chat messages for the given trip ID.
///
/// 5. **GET `/chat/{trip_id}`:**
///    - Extracts the `trip_id` from the URL path.
///    - Checks if any messages exist for the given trip ID via the `check_if_messages` function.
///        - If messages exist, retrieves them via the `get_messages` function and returns as a JSON response.
///        - Otherwise, returns a response with "No messages yet".
///
/// 6. **Fallback:**
///    If no route matches, returns a `Response::error("Not Found", 404)`.
///
/// # Notes
/// - Handlers like `index`, `input`, `get_trip`, `chat`, `check_if_messages`, and `get_messages` must be properly implemented.
/// - The included `chat.html` file is assumed to exist at `../public/chat.html`.
/// - The function is designed for asynchronous execution and leverages the `async` Rust programming model.
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response>{
    let path = req.path();

    if req.method() == Method::Get && path == "/" {
        return index().await;
    }
    else if req.method() == Method::Post && path == "/input"{
        return input(req, env, _ctx).await;
    }
    if req.method() == Method::Get && path.starts_with("/trip/") {
        let trip_id = path.trim_start_matches("/trip/").to_string();
        let accept_header = req.headers().get("Accept").unwrap_or_default().unwrap_or_default();
        if accept_header.contains("text/html") {
            let html = include_str!("../public/chat.html");
            return Ok(Response::from_html(html)?);
        } else {
            return get_trip(env, trip_id).await;
        }
    }
    if req.method() == Method::Post && path.starts_with("/trip/") {
        return chat(req, env, _ctx).await
    }
    if req.method() == Method::Get && path.starts_with("/chat/") {
        let trip_id = path.trim_start_matches("/chat/").to_string();
        if check_if_messages(trip_id.clone(), env.clone()).await? {
            let messages = get_messages(trip_id, env).await?;
            let body = serde_json::to_string(&messages)?;
            return Response::ok(body);
        }
        return Response::ok("No messages yet");
    }
    Response::error("Not Found", 404)
}

/// Handles an HTTP request to facilitate a chat interaction between a user and an AI.
///
/// # Arguments
/// * `req` - The HTTP request that contains the form data and any necessary metadata.
/// * `env` - The `Env` object, providing access to environment variables and external services.
/// * `_ctx` - Context parameter, not utilized in this implementation, but included for compatibility.
///
/// # Returns
/// Returns an `Ok(Response)` containing the AI's chat response if successful. Returns an error if
/// any required fields are missing or if an operation fails.
///
/// # Behavior
/// 1. Extracts the form data from the request, specifically looking for a `message` field.
///    - If the `message` field is missing, returns a `400 Missing field` error.
/// 2. Extracts the `trip_id` from the request path by removing the "/trip/" prefix.
/// 3. Creates a user message in the database by calling `create_message`, associating it with the trip and storing it as a "User" message.
///    - Returns an error if the database operation fails.
/// 4. Retrieves the current state of the trip by calling `get_trip`.
/// 5. Checks whether messages for the trip already exist by calling `check_if_messages`.
///    - If no prior messages are found, initiates an AI response with an empty message history.
///    - If messages are found, fetches the message history and includes it in the AI response generation.
/// 6. Delegates to the AI system by calling `ai::chat` to generate a response based on the message history and the user's message.
/// 7. Stores the AI response as a message in the database by calling `create_message` as an "AI" message.
///    - Returns an error if the database operation fails during this step.
/// 8. Returns an `Ok(Response)` containing the AI-generated response to the client.
///
/// # Errors
/// This function can return errors in the following scenarios:
/// - The "message" field is missing from the request's form data.
/// - Database operations (`create_message`, `get_trip`, `check_if_messages`) fail.
/// - AI response generation (`ai::chat`) fails.
///
/// # Example
/// ```
/// // Example HTTP request with "message" in form data
/// let req = Request::new().form_data("message", "Hello, AI!");
/// let response = chat(req, env, _ctx).await;
/// ```
///
/// This example demonstrates handling a user's "Hello, AI!" message in chat and returning the AI's response.
async fn chat(mut req: Request, env: Env, _ctx: Context) -> Result<Response>{
    let form = req.form_data().await?;
    let Some(FormEntry::Field(message)) = form.get("message") else {
        return Response::error("Missing field: message", 400);
    };
    let path = req.path();
    let trip_id = path.trim_start_matches("/trip/").to_string();
    create_message(trip_id.clone(), &message, "User", env.clone()).await.map_err(|e| Error::RustError(format!("db::create_message failed: {e}")))?;
    let mut trip = get_trip(env.clone(), trip_id.clone()).await?;
    if !check_if_messages(trip_id.clone(), env.clone()).await? {
        let resp = ai::chat(&env, &trip.text().await?, vec![("".to_string(),"".to_string(),"".to_string())], &message).await?;
        return Response::ok(resp);
    }
    let resp = ai::chat(&env, &trip.text().await?, get_messages(trip_id.clone(), env.clone()).await?, &message).await?;
    create_message(trip_id, &resp, "AI", env.clone()).await.map_err(|e| Error::RustError(format!("db::create_message failed: {e}")))?;
    Response::ok(resp)
}

/// Handles the `input` endpoint for creating a trip plan. This function is responsible for:
/// 1. Parsing and validating form data.
/// 2. Generating a unique trip ID.
/// 3. Interacting with the AI service to generate a travel plan.
/// 4. Initializing a corresponding durable object to store session-specific trip data.
/// 5. Creating a trip record in the database with its corresponding plan.
/// 6. Redirecting the user to the newly created trip's page.
///
/// # Parameters
/// - `req`: The incoming request containing form data with `destination` and `days` fields.
/// - `env`: The environment context providing required bindings (e.g., Durable Object, KV, AI services).
/// - `_ctx`: Execution context, not used in this implementation.
///
/// # Returns
/// `Result<Response>`:
/// - On success, a HTTP redirect response to the new trip's page.
/// - On failure, an error response with an appropriate status code and message.
///
/// # Errors
/// - Returns a `400 Bad Request` response:
///   - If the `destination` or `days` fields are missing in the form data.
///   - If the `days` field is not a valid number.
/// - Returns a `500 Internal Server Error` response:
///   - If the AI service fails to generate a trip plan.
///   - If the durable object initialization fails.
///   - If the database creation of the trip or its plan fails.
///   - If any other unexpected error occurs during the request lifecycle.
///
/// # Process Flow
/// 1. Parse form data and validate the presence of the `destination` and `days` fields.
/// 2. Parse the `days` value to ensure it is a valid number.
/// 3. Generate a new unique trip ID using `Uuid`.
/// 4. Establish a reference to the durable object using this trip ID.
/// 5. Call the `ai::create_plan` function with the destination and days to generate a travel plan.
/// 6. Create a `TripInit` payload with the generated plan and initialize the trip session durable object:
///    - Send a POST request to the durable object at the `https://trip-session/init` endpoint.
///    - If the request fails, return an error response.
/// 7. Store the trip data by calling `create_trip` to persist the trip in the database.
/// 8. Store the AI-generated plans with `db::create_plan` in the database.
/// 9. Build a redirect URL pointing to the new trip's page and return a `302 Redirect` response.
///
/// # Example
/// When called with valid form data (`destination="Paris"`, `days="5"`), the function:
/// - Creates a unique trip ID such as `12345678-abcd-1234-efgh-123456abcdef`.
/// - Generates an AI travel plan for Paris for 5 days.
/// - Initializes a trip session durable object and persists the trip to a database.
/// - Redirects the user to `/trip/12345678-abcd-1234-efgh-123456abcdef`.
async fn input(mut req: Request, env: Env, _ctx: Context) -> Result<Response>{
    let form = req.form_data().await?;
    let Some(FormEntry::Field(destination)) = form.get("destination") else {
        return Response::error("Missing field: destination", 400);
    };
    let Some(FormEntry::Field(days_str)) = form.get("days") else {
        return Response::error("Missing field: days", 400);
    };
    let days: u32 = days_str.parse().map_err(|_| Error::RustError("days must be a number".into()))?;
    let trip_id = Uuid::new_v4().to_string();
    let ns = env.durable_object("TRIP_SESSION_DO")?;
    let stub = ns.get_by_name(trip_id.as_str())?;

    let response = ai::create_plan(&env, &destination, days).await.map_err(|e| Error::RustError(format!("ai::create_plan failed: {e}")))?;
    let r = response.0.clone();
    let init_payload = TripInit { destination, days, response: r };

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    let mut init = RequestInit::new();
    init.method = Method::Post;
    init.with_headers(headers);
    init.with_body(Some(serde_json::to_string(&init_payload)?.into()));

    let do_req = Request::new_with_init("https://trip-session/init", &init)?;
    let mut resp = stub.fetch_with_request(do_req).await?;
    if resp.status_code() != 200 {
        let body = resp.text().await.unwrap_or_else(|_| "<no body>".into());
        return Response::error(format!("failed to initialize trip: {body}"), 500);
    }

    let trip = &TripData {
        id: trip_id.clone(),
        destination: init_payload.destination,
        days: init_payload.days,
    };
    create_trip(trip.clone(), env.clone()).await.map_err(|e| Error::RustError(format!("db::create_trip failed: {e}")))?;
    db::create_plan(trip.id.clone(),&response.0, &response.1, env.clone()).await.map_err(|e| Error::RustError(format!("db::create_plan failed: {e}")))?;
    let mut url = req.url()?;
    url.set_path(&format!("/trip/{trip_id}"));
    url.set_query(None);
    let resp = Response::redirect(url)?;
    Ok(resp)
}

/// Fetches a trip session from a durable object based on the provided trip ID.
///
/// # Arguments
/// * `env` - The `Env` object, which is typically used to access environment bindings
///   such as durable objects.
/// * `trip_id` - A `String` representing the unique identifier for the trip session
///   to be fetched.
///
/// # Returns
/// * `Result<Response>` - Returns an `Ok(Response)` if the fetch operation is successful,
///   or an `Err` if an error occurs during the process.
///
/// # Functionality
/// 1. Retrieves a reference to the `TRIP_SESSION_DO` durable object binding using `env.durable_object("TRIP_SESSION_DO")`.
/// 2. Generates an ID from the trip name (`trip_id`) using the `id_from_name` method.
/// 3. Retrieves a durable object stub using the trip ID.
/// 4. Constructs a `GET` request to the specific durable object endpoint (`https://trip-session/`).
/// 5. Sends the request to the durable object using the `stub.fetch_with_request` method.
/// 6. Returns the HTTP response from the durable object.
///
/// # Errors
/// This function may return an error in the following cases:
/// * If the durable object binding "TRIP_SESSION_DO" is not found.
/// * If the `trip_id` cannot be converted to a valid durable object ID.
/// * If there is an issue creating or sending the `Request`.
/// * If there is an issue while fetching the response from the durable object.
///
/// # Example Usage
/// ```rust
/// use worker::{Env, Response};
///
/// async fn example(env: Env) -> Result<Response> {
///     let trip_id = "some-trip-id".to_string();
///     let response = get_trip(env, trip_id).await?;
///     Ok(response)
/// }
/// ```
///
/// Ensure that your Worker has the `TRIP_SESSION_DO` binding configured in the environment for the function to work properly.
async fn get_trip(env: Env, trip_id: String) -> Result<Response>{
    let ns = env.durable_object("TRIP_SESSION_DO")?;

    let stub = ns.get_by_name(trip_id.as_str());

    let mut init = RequestInit::new();
    init.method = Method::Get;

    let do_req = Request::new_with_init("https://trip-session/", &init)?;
    let resp = stub?.fetch_with_request(do_req).await?;

    Ok(resp)
}

/// Serves the HTML content for the application's index page.
///
/// This asynchronous function reads an HTML file located in the `../public` directory
/// and serves it as the response with proper `Content-Type` headers set to `text/html; charset=utf-8`.
///
/// # Returns
/// - `Ok(Response)` containing the HTML content to be served as the response if successful.
/// - `Err` if an error occurs while creating the response or setting the headers.
///
/// # Errors
/// This function can return an error in the following cases:
/// - If the response creation from the HTML content fails.
/// - If the `Content-Type` header cannot be set properly.
///
/// # Example
/// ```rust
/// let response = index().await?;
/// ```
async fn index() -> Result<Response>{
    let html = include_str!("../public/index.html");
    let mut resp = Response::from_html(html)?;
    resp.headers_mut()
        .set("Content-Type", "text/html; charset=utf-8")?;
    Ok(resp)
}

/// The `TripSession` struct is a durable object enabling state persistence and concurrency handling across multiple instances.
///
/// # Attributes:
/// - `state`: A `State` object that represents the persistent storage and state management for the `TripSession`.
///
/// # Durable Object:
/// This struct is marked with the `#[durable_object]` attribute, which allows the object to:
/// - Preserve state between multiple executions.
/// - Handle concurrent requests or operations in a consistent manner.
/// - Be used in systems where long-lived, distributed state management is required.
///
/// # Use Case:
/// This durable object can be utilized in applications requiring session-based data persistence,
/// such as managing trip or ride information, storing trip session metadata, or tracking progress
/// in real-time.
///
/// Example integration includes the use of `state` to persist details like trip status, active users,
/// or associated session data across instances of the `TripSession`.
#[durable_object]
pub struct TripSession{
    state: State,
}

impl DurableObject for TripSession{
    /// Creates a new instance of the containing type with the provided `state`.
    ///
    /// # Parameters
    /// - `state`: The `State` object used to initialize the instance.
    /// - `_`: An unused `Env` parameter, typically passed in but ignored in this function.
    ///
    /// # Returns
    /// A new instance of the type initialized with the given `state`.
    ///
    /// # Example
    /// ```
    /// let state = State::new();
    /// let env = Env::new();
    /// let instance = YourType::new(state, env);
    /// ```
    fn new(state: State, _: Env) -> Self{ Self { state }}

    /// Handles incoming HTTP requests and performs various operations based on the request.
    ///
    /// This function performs the following actions depending on the HTTP method and path:
    ///
    /// - **POST /init**:
    ///   This endpoint is used to initialize or overwrite the state of the Durable Object (DO).
    ///   It expects a JSON body (`TripInit`) containing:
    ///     - `destination`: A string that represents the destination.
    ///     - `days`: A u32 representing the number of days.
    ///     - `response`: A string that holds additional response data.
    ///   The data is stored persistently in the DO's storage. On success, responds with:
    ///     - HTTP 200 OK, with the message `"initialized"`.
    ///
    /// - **GET /**:
    ///   This endpoint retrieves the initialized trip data stored in the DO's state.
    ///   It fetches the following keys from DO's storage:
    ///     - `destination`: The stored destination (`String`).
    ///     - `days`: The stored number of days (`u32`).
    ///     - `response`: The stored response (`String`).
    ///   If all keys (`destination`, `days`, and `response`) are found, it constructs a JSON response like:
    ///   ```json
    ///   {
    ///       "destination": "string",
    ///       "days": 7,
    ///       "response": "string"
    ///   }
    ///   ```
    ///   Responds with HTTP 200 OK and returns the JSON payload.
    ///   If any key is missing, responds with:
    ///     - HTTP 404 Not Found, with the message `"trip not initialized"`.
    ///
    /// - All Other Requests:
    ///   For any other HTTP methods or paths, responds with:
    ///     - HTTP 404 Not Found, with the message `"not found"`.
    ///
    /// ### Arguments
    /// - `&self`: Reference to the current instance of the Durable Object.
    /// - `req`: The incoming HTTP `Request` that contains the HTTP method, URL, headers, and body.
    ///
    /// ### Returns
    /// - A `Result<Response>` which:
    ///   - On success, is a `Response` object containing either a JSON payload or a message.
    ///   - On failure, responds with the appropriate HTTP error.
    ///
    /// ### Errors
    /// - Returns HTTP 404 Not Found for unrecognized paths or methods.
    /// - Returns HTTP 404 for GET requests if the trip data is not initialized.
    /// - May return errors if there are issues fetching the request payload or interacting with Durable Object storage.
    ///
    /// ### Example
    /// #### POST /init
    /// ```json
    /// {
    ///     "destination": "Paris",
    ///     "days": 5,
    ///     "response": "Ready to travel"
    /// }
    /// ```
    /// Response:
    /// ```text
    /// HTTP 200 OK
    /// initialized
    /// ```
    ///
    /// #### GET /
    /// Response when trip is initialized:
    /// ```json
    /// {
    ///     "destination": "Paris",
    ///     "days": 5,
    ///     "response": "Ready to travel"
    /// }
    /// ```
    /// Response when trip is not initialized:
    /// ```text
    /// HTTP 404 Not Found
    /// trip not initialized
    /// ```
    async fn fetch(&self, mut req: Request) -> Result<Response> {
        let url = req.url()?;
        let pathname = url.path();

        if req.method() == Method::Post && pathname == "/init" {
            // Initialize or overwrite this DO's state
            let init: TripInit = req.json().await?;
            self.state.storage().put("destination", &init.destination).await?;
            self.state.storage().put("days", &init.days).await?;
            self.state.storage().put("response", &init.response).await?;
            return Response::ok("initialized");
        }

        if req.method() == Method::Get && pathname == "/" {
            let destination: Option<String> = self.state.storage().get("destination").await?;
            let days: Option<u32> = self.state.storage().get("days").await?;
            let response: Option<String> = self.state.storage().get("response").await?;
            if let (Some(destination), Some(days), Some(response)) = (destination, days, response) {
                // Use the DO's own id as the trip id for round-tripping if you like
                let data = serde_json::json!({
                    "destination": destination,
                    "days": days,
                    "response": response
                });
                return Response::from_json(&data);
            } else {
                return Response::error("trip not initialized", 404);
            }
        }

        Response::error("not found", 404)
    }
}