use coalescing::coalescing_service_server::{CoalescingService, CoalescingServiceServer};
use coalescing::{Request as GrpcRequest, Response as GrpcResponse};
use scylla::{Session, SessionBuilder};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tonic::{transport::Server, Request, Response, Status};
pub mod coalescing {
    tonic::include_proto!("coalescing");
}

enum Message {
    Request(GrpcRequest, oneshot::Sender<GrpcResponse>),
    Completed(GrpcRequest, GrpcResponse),
}

async fn shard_task(mut receiver: mpsc::Receiver<Message>) {
    let mut map: HashMap<i64, Vec<oneshot::Sender<GrpcResponse>>> = HashMap::new();

    while let Some(msg) = receiver.recv().await {
        match msg {
            Message::Request(req, resp_sender) => {
                let entry = map.entry(req.id).or_insert_with(Vec::new);
                entry.push(resp_sender);
            }
            Message::Completed(req, resp) => {
                if let Some(waiters) = map.remove(&req.id) {
                    for waiter in waiters {
                        let _ = waiter.send(resp.clone());
                    }
                }
            }
        }
    }
}

pub struct MyCoalescingService {
    sender: mpsc::Sender<Message>,
    session: Arc<Session>,
}

#[tonic::async_trait]
impl CoalescingService for MyCoalescingService {
    async fn get_response(
        &self,
        request: Request<GrpcRequest>,
    ) -> Result<Response<GrpcResponse>, Status> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        let grpc_req = request.into_inner();
        let sender = self.sender.clone();
        let req_clone = grpc_req.clone();
        let session = Arc::clone(&self.session);
        tokio::spawn(async move {
            let result = match read_from_database(&grpc_req.id, session).await {
                Ok(result) => result,
                Err(e) => {
                    println!("Error reading from database: {:?}", e);
                    let error_resp = GrpcResponse {
                        channel_id: -1,
                        message_id: -1,
                        bucket: -1,
                        content: format!("Error: {:?}", e),
                        author_id: -1,
                    };
                    sender
                        .send(Message::Completed(grpc_req.clone(), error_resp))
                        .await
                        .unwrap();
                    return;
                }
            };
            let resp = GrpcResponse {
                channel_id: result.channel_id,
                message_id: result.message_id,
                bucket: result.bucket,
                content: result.content,
                author_id: result.author_id,
            };
            sender
                .send(Message::Completed(grpc_req.clone(), resp))
                .await
                .unwrap();
        });
        self.sender
            .send(Message::Request(req_clone, oneshot_sender))
            .await
            .unwrap();
        let response = oneshot_receiver.await.unwrap();
        Ok(Response::new(response))
    }
    async fn without_coalescing(
        &self,
        request: Request<GrpcRequest>,
    ) -> Result<Response<GrpcResponse>, Status> {
        let grpc_req = request.into_inner();
        let session = Arc::clone(&self.session);
        let result = match read_from_database(&grpc_req.id, session).await {
            Ok(result) => result,
            Err(e) => {
                println!("Error reading from database: {:?}", e);
                let error_resp = GrpcResponse {
                    channel_id: -1,
                    message_id: -1,
                    bucket: -1,
                    content: format!("Error: {:?}", e),
                    author_id: -1,
                };
                error_resp
            }
        };
        let resp = GrpcResponse {
            channel_id: result.channel_id,
            message_id: result.message_id,
            bucket: result.bucket,
            content: result.content,
            author_id: result.author_id,
        };
        Ok(Response::new(resp))
    }
}

async fn read_from_database(
    id: &i64,
    session: Arc<Session>,
) -> Result<GrpcResponse, Box<dyn std::error::Error + Send + Sync>> {
    let query = "SELECT * FROM messages_keyspace.messages WHERE message_id = ? AND channel_id = 23 AND bucket = 1 LIMIT 1";
    let rows = session
        .query(query, (id,))
        .await?
        .rows
        .ok_or("There were no rows found with given id")?;
    let row = rows.get(0).ok_or("No rows for given id")?;
    let channel_id: i64 = row.columns[0].as_ref().unwrap().as_bigint().unwrap() as i64;
    let message_id: i64 = row.columns[2].as_ref().unwrap().as_bigint().unwrap() as i64;
    let bucket: i32 = row.columns[1].as_ref().unwrap().as_int().unwrap() as i32;
    let content: String = row.columns[4]
        .as_ref()
        .unwrap()
        .as_text()
        .unwrap()
        .to_string() as String;
    let author_id: i64 = row.columns[3].as_ref().unwrap().as_bigint().unwrap() as i64;

    let res = GrpcResponse {
        channel_id,
        message_id,
        bucket,
        content,
        author_id,
    };
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("service running");
    let uri = env::var("DB_ADDRESS").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let w_session: Session = SessionBuilder::new().known_node(uri).build().await?;
    let session = Arc::new(w_session);
    let (sender, receiver) = mpsc::channel(1024);
    let my_coalescing_service = MyCoalescingService { sender, session };
    // todo
    // implement scaling with shardtasks - amount of shard tasks depends on
    // amount of cpus assigned in k8s for example and with that change we can perform
    // modulo hashin on request id
    tokio::spawn(async move {
        shard_task(receiver).await;
    });
    let addr = "127.0.0.1:50051".parse().unwrap();
    Server::builder()
        .add_service(CoalescingServiceServer::new(my_coalescing_service))
        .serve(addr)
        .await?;

    Ok(())
}
