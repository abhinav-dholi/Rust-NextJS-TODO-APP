// Import necessary crates
use actix_cors::Cors; // Cross-Origin Resource Sharing (CORS) middleware
use actix_web::{web, App, HttpResponse, HttpServer, Responder}; // Actix Web framework for building web applications
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize}; // Serialization and deserialization for JSON payloads
use std::sync::Mutex; // Mutex for safe concurrent access to shared state
use uuid::Uuid; // Universally Unique Identifier (UUID) for unique todo item IDs // Chrono for handling timestamps

// Define a struct for a single To-Do item
#[derive(Serialize, Deserialize, Clone)]
struct TodoItem {
    id: Uuid,                          // Unique identifier for the to-do item
    title: String,                     // Title or description of the to-do task
    completed: bool,                   // Status of the task (true if completed, false otherwise)
    created_at: DateTime<Utc>,         // Timestamp for when the task was created
    updated_at: Option<DateTime<Utc>>, // Optional timestamp for when the task was last updated
}

// Struct for handling create to-do request payload
#[derive(Deserialize)]
struct CreateTodoItem {
    title: String,   // Title of the new to-do task
    completed: bool, // Initial status of the task
}

// Struct for handling update to-do request payload
#[derive(Deserialize)]
struct UpdateTodoItem {
    title: Option<String>,   // Optional new title for the task
    completed: Option<bool>, // Optional new completion status
}

// Application state shared across multiple requests
struct AppState {
    todo_list: Mutex<Vec<TodoItem>>, // Mutex-protected vector to store to-do items safely across multiple threads
}

// Asynchronous function to handle GET requests for fetching to-do items
async fn get_todos(data: web::Data<AppState>) -> impl Responder {
    // Lock the mutex to safely access the shared state across multiple threads
    let todos = data.todo_list.lock().unwrap();

    // Return an HTTP response with the list of to-do items serialized as JSON
    HttpResponse::Ok().json(&*todos)
}

// Asynchronous function to handle POST requests for creating a new to-do item
async fn add_todo(item: web::Json<CreateTodoItem>, data: web::Data<AppState>) -> impl Responder {
    let mut todos = data.todo_list.lock().unwrap(); // Lock the mutex to safely access the shared state
    let new_todo = TodoItem {
        id: Uuid::new_v4(),        // Generate a new UUID for the to-do item
        title: item.title.clone(), // Set the title of the to-do item
        completed: item.completed, // Set the completion status of the to-do item
        created_at: Utc::now(),    // Set the creation timestamp of the to-do item
        updated_at: None,          // Set the update timestamp to None initially
    };
    todos.push(new_todo); // Add the new to-do item to the list
    HttpResponse::Ok().json(todos.clone()) // Return an HTTP response with the updated list of to-do items
}

async fn update_todo(
    path: web::Path<Uuid>,
    item: web::Json<UpdateTodoItem>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut todos = data.todo_list.lock().unwrap();
    if let Some(todo) = todos.iter_mut().find(|todo| todo.id == *path) {
        if let Some(title) = &item.title {
            todo.title = title.clone();
        }
        if let Some(completed) = item.completed {
            todo.completed = completed;
        }
        todo.updated_at = Some(Utc::now());
        HttpResponse::Ok().json(todo.clone())
    } else {
        HttpResponse::NotFound().body("Todo item not found")
    }
}

async fn delete_todo(path: web::Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
    let mut todos = data.todo_list.lock().unwrap();
    if todos.iter().any(|todo| todo.id == *path) {
        todos.retain(|todo| todo.id != *path);
        HttpResponse::Ok().json(todos.clone())
    } else {
        HttpResponse::NotFound().body("Todo item not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        todo_list: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

    App::new()
        .app_data(app_state.clone())
        .wrap(cors)
        .route("/todos", web::get().to(get_todos))
        .route("/todos", web::post().to(add_todo))
        .route("/todos/{id}", web::put().to(update_todo))
        .route("/todos/{id}", web::delete().to(delete_todo))
    })
    .bind("127.0.0.1:8080") ? .run().await // ? is for error handling in rust (reminder)
}