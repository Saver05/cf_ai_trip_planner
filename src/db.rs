use worker::*;
use worker::wasm_bindgen::__rt::IntoJsResult;
use crate::TripData;


/// Asynchronously creates a new trip entry in the "TripPlanner" database.
///
/// # Description
/// This function inserts a new trip record into the `trips` table of the "TripPlanner" D1 database.
/// It prepares an SQL statement to insert the trip details (id, destination, and days) and executes
/// it as part of a batch operation. The function ensures that the trip creation was successful before
/// returning the result.
///
/// # Arguments
/// * `trip` - A `TripData` object containing the trip details:
///   - `id`: The unique identifier for the trip.
///   - `destination`: The destination of the trip.
///   - `days`: The number of days for the trip.
/// * `env` - An `Env` object used to access the "TripPlanner" D1 database.
///
/// # Returns
/// A `Result<D1Result>` which, on success, contains the result of the database operation. If an error
/// occurs, it returns an `Error` variant with a descriptive error message.
///
/// # Errors
/// This function can return an `Err` for the following reasons:
/// - If there is an issue accessing the "TripPlanner" database.
/// - If preparing or binding the SQL statement fails.
/// - If the batch operation fails to execute.
/// - If the database operation does not succeed (e.g., due to constraint violations).
///
/// # Example
/// ```
/// use your_crate::{create_trip, TripData, Env};
///
/// #[tokio::main]
/// async fn main() {
///     let trip = TripData {
///         id: "trip_123".to_string(),
///         destination: "Paris".to_string(),
///         days: 5,
///     };
///
///     let env = Env::new(); // Assume `Env` is properly initialized
///
///     match create_trip(trip, env).await {
///         Ok(result) => println!("Trip created successfully: {:?}", result),
///         Err(e) => eprintln!("Error creating trip: {}", e),
///     }
/// }
/// ```
///
/// # Notes
/// - Ensure the `TripData` structure and `Env` environment are properly defined and initialized.
/// - The database schema for the `trips` table should match the expected fields (`id`, `destination`, `days`).
/// - Exception handling is implemented to ensure meaningful error messages in case of failures.
pub async fn create_trip(trip: TripData, env: Env) -> Result<D1Result>{
    let db = env.d1("TripPlanner")?;

    let statement = db.prepare("INSERT INTO trips (id, destination, days) VALUES (?, ?, ?)")
        .bind(&[trip.id.into_js_result()?,trip.destination.into_js_result()?,trip.days.into_js_result()?])?;
    let result = db.batch(vec![statement]).await?;
    let mut iter_result = result.into_iter();
    if let Some(r) = iter_result.next(){
        if !r.success(){
            return Err(Error::RustError(format!("Failed to create trip with error {}",r.error().unwrap())));
        }
        Ok(r)
    }
    else{
        Err(Error::RustError("Failed to create trip".into()))
    }
}

/// Asynchronously creates a new plan for a specific trip in the database.
///
/// # Arguments
///
/// * `trip_id` - A `String` that represents the unique identifier for the trip.
/// * `plan` - A reference to a `String` that represents the plan details to be saved.
/// * `input_text` - A reference to a `String` containing additional input text related to the plan.
/// * `env` - The `Env` object containing the environment configuration and database access.
///
/// # Returns
///
/// Returns a `Result<D1Result, Error>` object:
/// - On success: Returns a `D1Result` object indicating that the plan has been successfully created.
/// - On failure: Returns an `Error` explaining why the creation of the plan failed.
///
/// # Errors
///
/// This function can return the following errors:
/// - `RustError`: If there is an issue binding the values or executing the prepared SQL statement.
/// - `RustError`: If the underlying database operation is not successful.
///
/// # Behavior
///
/// 1. Establishes a connection to the `TripPlanner` database from the provided `Env`.
/// 2. Generates the current timestamp using the `Date::now()` function.
/// 3. Prepares an SQL `INSERT` statement to store the new plan with the `trip_id`, `plan`, `input_text`,
///    and the current timestamp.
/// 4. Executes the SQL statements in batch mode.
/// 5. Evaluates the database operation result to ensure the plan was created successfully:
///     - If successful, returns the corresponding `D1Result`.
///     - If there is a failure, returns an appropriate error (e.g., a `RustError` with details).
///
/// # Example
///
/// ```rust
/// use crate::{create_plan, Env, D1Result};
///
/// #[tokio::main]
/// async fn main() {
///     let trip_id = "trip123".to_string();
///     let plan = "Visit Paris attractions".to_string();
///     let input_text = "Eiffel Tower, Louvre Museum".to_string();
///     let env = Env::new();
///
///     match create_plan(trip_id, &plan, &input_text, env).await {
///         Ok(result) => println!("Plan created successfully: {:?}", result),
///         Err(e) => eprintln!("Failed to create plan: {:?}", e),
///     }
/// }
/// ```
pub async fn create_plan(trip_id: String, plan: &String, input_text: &String, env: Env) -> Result<D1Result>{
    let db = env.d1("TripPlanner")?;
    let date = Date::now();
    let timestamp = date.to_string();
    let statement = db.prepare("INSERT INTO plans (trip_id, plan, input_text, updated_at) VALUES (?,?,?,?)")
        .bind(&[trip_id.into_js_result()?,plan.into_js_result()?,input_text.into_js_result()?,timestamp.into_js_result()?])?;
    let result = db.batch(vec![statement]).await?;
    let mut iter_result = result.into_iter();
    if let Some(r) = iter_result.next(){
        if !r.success(){
            return Err(Error::RustError(format!("Failed to create plan with error {}",r.error().unwrap())));
        }
        Ok(r)
    }
    else{
        Err(Error::RustError("Failed to create plan".into()))
    }
}

/// Asynchronous function to create a new message entry in the database for a specific trip.
///
/// # Parameters
/// - `trip_id`: A `String` that represents the unique identifier of the trip to which the message belongs.
/// - `message`: A reference to a `String` containing the content of the message.
/// - `messager_role`: A `&str` specifying the role of the message sender (e.g., "admin", "user").
/// - `env`: An `Env` object used to interact with the environment and database.
///
/// # Returns
/// - On success: A `Result<D1Result>` containing a successful database operation result.
/// - On failure: A `Result<D1Result>` with an `Err` variant, encapsulating an error message if the insertion fails.
///
/// # Errors
/// - Returns an error if the environment cannot access the `TripPlanner` database.
/// - Returns an error if preparing the database statement or binding parameters fails.
/// - Returns an error if the database operation (`batch`) fails to execute successfully or if no response is received.
///
/// # Database Details
/// - Table: `messages`
/// - Columns:
///   1. `trip_id` - Unique identifier for the trip (provided as input).
///   2. `message` - The content of the message (provided as input).
///   3. `messager_role` - Role of the sender (provided as input).
///   4. `created_at` - The timestamp when the message is created (automatically generated using `Date::now()`).
///
/// # Example Usage
/// ```rust
/// let result = create_message(
///     "trip123".to_string(),
///     &"Hello, your trip is confirmed!".to_string(),
///     "admin",
///     env,
/// ).await;
/// match result {
///     Ok(res) => println!("Message created successfully: {:?}", res),
///     Err(err) => eprintln!("Failed to create message: {:?}", err),
/// }
/// ```
///
/// # Notes
/// - The function binds the input values (`trip_id`, `message`, `messager_role`, and `created_at`) to an SQL `INSERT` query.
/// - Uses a batched database operation for efficient execution.
/// - Ensures error handling for both database interaction and result validation.
pub async fn create_message(trip_id: String, message: &String, messager_role: &str, env: Env) -> Result<D1Result>{
    let db = env.d1("TripPlanner")?;
    let date = Date::now();
    let timestamp = date.to_string();
    let statement = db.prepare("INSERT INTO messages (trip_id, message, messager_role, created_at) VALUES (?,?,?,?)")
        .bind(&[trip_id.into_js_result()?,message.into_js_result()?,messager_role.into_js_result()?,timestamp.into_js_result()?])?;
    let result = db.batch(vec![statement]).await?;
    let mut iter_result = result.into_iter();
    if let Some(r) = iter_result.next(){
        if !r.success(){
            return Err(Error::RustError(format!("Failed to create message with error {}",r.error().unwrap())));
        }
        Ok(r)
    }
    else{
        Err(Error::RustError("Failed to create message".into()))
    }
}

/// Asynchronously checks if there are any messages associated with a given trip ID in the database.
///
/// This function queries the "messages" table in the "TripPlanner" database to determine if there are
/// any records corresponding to the provided `trip_id`. It returns `true` if at least one message
/// exists for the specified trip ID, and `false` otherwise.
///
/// # Arguments
///
/// * `trip_id` - A `String` representing the unique identifier of the trip to check for associated messages.
/// * `env` - An `Env` object that provides access to the database environment configuration.
///
/// # Returns
///
/// Result containing:
/// * `Ok(bool)` - `true` if messages exist for the given `trip_id`, `false` if no messages exist.
/// * `Err` - If any error occurs during database interaction or query execution.
///
/// # Errors
///
/// This function will return an error in the following cases:
/// - Unable to access the "TripPlanner" database through the provided `env`.
/// - Failure to prepare the SQL query or bind the `trip_id` parameter.
/// - Issues during the query execution or in extracting the result.
///
/// # Example
///
/// ```rust
/// use some_crate::check_if_messages;
/// use some_crate::Env;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let trip_id = "12345".to_string();
///     let env = Env::new(); // Assume this initializes the environment properly.
///
///     let has_messages = check_if_messages(trip_id, env).await?;
///     
///     if has_messages {
///         println!("There are messages for the provided trip ID.");
///     } else {
///         println!("No messages found for the provided trip ID.");
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Notes
///
/// - The query used in this function limits the number of rows retrieved to 1 for efficiency.
/// - This function uses the `d1` method and expects the `Env` object to provide access to the database named "TripPlanner".
/// - The result is parsed as `serde_json::Value` type to determine if any record exists.
///
/// # Dependencies
///
/// This function assumes the following libraries or crates are available:
/// - `async`/`await` for asynchronous operation.
/// - `serde_json::Value` for handling database query results.
/// - Database access methods compatible with `Env` and `d1`.
pub async fn check_if_messages(trip_id: String, env: Env) -> Result<bool> {
    let db = env.d1("TripPlanner")?;
    let statement = db.prepare("SELECT 1 as one FROM messages WHERE trip_id = ? LIMIT 1")
        .bind(&[trip_id.into_js_result()?])?;
    let result = statement.first::<serde_json::Value>(None).await?;
    Ok(result.is_some())
}

/// Asynchronously retrieves a list of messages associated with a specific trip ID.
///
/// # Arguments
///
/// * `trip_id` - A `String` representing the unique identifier for the trip.
/// * `env` - An `Env` object that provides access to database and environment configuration.
///
/// # Returns
///
/// On success, returns a `Result` containing a `Vec` of tuples, where each tuple consists of:
/// - `String`: The message content.
/// - `String`: The role of the message sender (e.g., "user", "admin").
/// - `String`: The timestamp when the message was created.
///
/// On failure, returns an error indicating a failure in the database interaction or data retrieval.
///
/// # Errors
///
/// This function will return an error if:
/// - There is an issue connecting to the "TripPlanner" database.
/// - The SQL query fails to execute properly.
/// - The `trip_id` cannot be bound to the prepared SQL statement.
/// - The result conversion to expected JSON structure or data extraction fails.
///
/// # Example
///
/// ```rust
/// use some_module::{get_messages, Env};
///
/// #[tokio::main]
/// async fn main() {
///     let env = Env::new();
///     let trip_id = "12345".to_string();
///
///     match get_messages(trip_id, env).await {
///         Ok(messages) => {
///             for (message, role, created_at) in messages {
///                 println!("Message: {}, Role: {}, Created At: {}", message, role, created_at);
///             }
///         }
///         Err(e) => {
///             eprintln!("Failed to retrieve messages: {:?}", e);
///         }
///     }
/// }
/// ```
///
/// This function assumes that the `messages` table in the database includes the following columns:
/// - `message` (text content of the message),
/// - `messager_role` (role of the sender),
/// - `created_at` (timestamp of message creation).
///
pub async fn get_messages(trip_id: String, env: Env) -> Result<Vec<(String, String, String)>> {
    let db = env.d1("TripPlanner")?;
    let statement = db.prepare("SELECT message, messager_role, created_at FROM messages WHERE trip_id = ? ")
        .bind(&[trip_id.into_js_result()?])?;
    let result = statement.all().await?;
    let messages = result
        .results::<serde_json::Value>()? // get as JSON-like rows
        .into_iter()
        .filter_map(|row| {
            Some((
                row.get("message")?.as_str()?.to_string(),
                row.get("messager_role")?.as_str()?.to_string(),
                row.get("created_at")?.as_str()?.to_string(),
            ))
        })
        .collect::<Vec<_>>();

    Ok(messages)
}