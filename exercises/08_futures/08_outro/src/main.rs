use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use once_cell::sync::Lazy;
use outro_08::data::{Ticket, TicketDraft, TicketPatch};
use outro_08::store::{TicketId, TicketStore};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

static TICKET_STORE: Lazy<Mutex<TicketStore>> = Lazy::new(|| Mutex::new(TicketStore::new()));

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", post(create_ticket))
        .route("/{id}", get(get_ticket).patch(update_ticket));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
struct CreateTicketResponse {
    id: TicketId,
}

async fn create_ticket(Json(payload): Json<TicketDraft>) -> Json<CreateTicketResponse> {
    let id = TICKET_STORE.lock().await.add_ticket(payload);

    Json(CreateTicketResponse { id })
}

async fn get_ticket(Path(id): Path<TicketId>) -> Result<Json<Ticket>, StatusCode> {
    let ticket = TICKET_STORE.lock().await.get(id).cloned();

    match ticket {
        Some(ticket) => Ok(Json(ticket)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_ticket(
    Path(id): Path<TicketId>,
    Json(patch): Json<TicketPatch>,
) -> Result<Json<Ticket>, StatusCode> {
    let mut store = TICKET_STORE.lock().await;

    let ticket = store.get_mut(id);
    match ticket {
        Some(ticket) => {
            if let Some(title) = patch.title {
                ticket.title = title;
            }
            if let Some(description) = patch.description {
                ticket.description = description;
            }
            if let Some(status) = patch.status {
                ticket.status = status;
            }

            Ok(Json(ticket.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
