// pub mod coalescing {
//     tonic::include_proto!("coalescing");
// }
// use coalescing::{Request, Response};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// pub struct JsonRequest {
//     id: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct JsonResponse {
//     pub id: String,
//     pub result: String,
// }

// impl From<Request> for JsonRequest {
//     fn from(proto_request: Request) -> Self {
//         JsonRequest {
//             id: proto_request.id,
//         }
//     }
// }

// impl From<JsonRequest> for Request {
//     fn from(json_request: JsonRequest) -> Self {
//         Request {
//             id: json_request.id,
//         }
//     }
// }

// impl From<Response> for JsonResponse {
//     fn from(proto_response: Response) -> Self {
//         JsonResponse {
//             id: proto_response.id,
//             result: proto_response.result,
//         }
//     }
// }

// impl From<JsonResponse> for Response {
//     fn from(json_response: JsonResponse) -> Self {
//         Response {
//             id: json_response.id,
//             result: json_response.result,
//         }
//     }
// }
