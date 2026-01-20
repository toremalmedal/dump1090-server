use flight::flight_service_server::{FlightService, FlightServiceServer};
use flight::{Aircraft, Empty, FlightData, HistoricalData, Receiver};
use http::{HeaderName, HeaderValue};
use reqwest::Method;
use serde_json::Value;
use tonic::{
    Request, Response, Status,
    transport::{Identity, Server, ServerTlsConfig},
};
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;

pub const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("dump1090-server_binary");
pub mod flight {
    tonic::include_proto!("dump1090_server");
}

fn parse_flight_data(data: Value) -> FlightData {
    let now = data["now"].as_f64().unwrap_or(0.0);
    let messages = data["messages"].as_i64().unwrap_or(0) as i32;

    let mut aircraft_vec = Vec::new();
    if let Some(aircrafts) = data["aircraft"].as_array() {
        for a in aircrafts {
            let aircraft = Aircraft {
                hex: a["hex"].as_str().unwrap_or("").to_string(),
                flight: a["flight"].as_str().map(|s| s.to_string()),
                lat: a["lat"].as_f64(),
                lon: a["lon"].as_f64(),
                track: a["track"].as_f64(),
                track_rate: a["track_rate"].as_f64(),
            };
            // Only include if hex exists
            if !aircraft.hex.is_empty() {
                aircraft_vec.push(aircraft);
            }
        }
    }

    FlightData {
        now,
        messages,
        aircraft: aircraft_vec,
    }
}

pub struct MyFlightService;

#[tonic::async_trait]
impl FlightService for MyFlightService {
    async fn get_flight_data(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<FlightData>, Status> {
        // Use provided json_dir or the test-data folder
        let json_dir = std::env::var("JSON_DIR").unwrap_or("./test-data".to_string());
        let aircraft_path = json_dir + "/aircraft.json";

        let file_content = tokio::fs::read_to_string(aircraft_path)
            .await
            .map_err(|e| Status::internal(format!("Failed to read JSON file: {}", e)))?;

        let data: Value = serde_json::from_str(&file_content)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        let flight_data = parse_flight_data(data);

        Ok(Response::new(flight_data))
    }

    async fn get_historical_data(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<HistoricalData>, Status> {
        // Use provided json_dir or the test-data folder
        let json_dir = std::env::var("JSON_DIR").unwrap_or("./test-data".to_string());

        let mut entries = tokio::fs::read_dir(json_dir).await?;

        let mut history = HistoricalData::default();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(p) = path.to_str()
                && p.contains("history")
            {
                let file_content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| Status::internal(format!("Failed to read JSON file: {}", e)))?;

                let data: Value = serde_json::from_str(&file_content)
                    .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

                let flight_data = parse_flight_data(data);
                history.flight_data.push(flight_data);
            }
        }
        Ok(Response::new(history))
    }

    async fn get_receiver_data(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Receiver>, Status> {
        // Use provided json_dir or the test-data folder
        let json_dir = std::env::var("JSON_DIR").unwrap_or("./test-data".to_string());
        let aircraft_path = json_dir + "/receiver.json";

        let file_content = tokio::fs::read_to_string(aircraft_path)
            .await
            .map_err(|e| Status::internal(format!("Failed to read JSON file: {}", e)))?;

        let data: Value = serde_json::from_str(&file_content)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        let version = data["version"].as_f64().unwrap_or(0.0);
        let refresh = data["refresh"].as_i64().unwrap_or(0) as i32;
        let history = data["history"].as_i64().unwrap_or(0) as i32;

        let receiver = Receiver {
            version,
            history,
            refresh,
        };

        Ok(Response::new(receiver))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_url = match std::env::var("GRPC_SERVER_URL") {
        Ok(url) => url,
        Err(e) => {
            panic!("Could not retrieve runtime env variable GRPC_SERVER_URL, got error:{e}");
        }
    };

    let addr = server_url.parse()?;
    let flight_service = MyFlightService;

    let flight_service_reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let allow_origin_domain = match std::env::var("ALLOW_ORIGIN") {
        Ok(domain) => domain.parse::<HeaderValue>().unwrap(),
        Err(_) => "*".parse::<HeaderValue>().unwrap(),
    };

    let cors = CorsLayer::new()
        .allow_headers([
            http::header::CONTENT_TYPE,
            HeaderName::from_static("x-grpc-web"),
        ])
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(allow_origin_domain);

    let cert_path = std::env::var("CERT_PATH");
    let key_path = std::env::var("KEY_PATH");

    // if cert and key exists in env variables, add tls layer
    let server = match (cert_path, key_path) {
        (Ok(c), Ok(k)) => {
            let cert = std::fs::read_to_string(c)?;
            let key = std::fs::read_to_string(k)?;
            let tls_config = ServerTlsConfig::new().identity(Identity::from_pem(&cert, &key));
            Server::builder()
                .accept_http1(true)
                .tls_config(tls_config)?
                .layer(cors)
                .layer(GrpcWebLayer::new())
                .add_service(flight_service_reflector)
                .add_service(FlightServiceServer::new(flight_service))
                .serve(addr)
        }
        _ => Server::builder()
            .accept_http1(true)
            .layer(cors)
            .layer(GrpcWebLayer::new())
            .add_service(flight_service_reflector)
            .add_service(FlightServiceServer::new(flight_service))
            .serve(addr),
    };

    println!("FlightService gRPC server listening on {}", addr);
    let json_dir = std::env::var("JSON_DIR").unwrap_or("./test-data".to_string());
    let aircraft_path = json_dir + "/aircraft.json";
    println!("Serving flight data from: {}", aircraft_path);

    let _ = server.await;

    Ok(())
}
