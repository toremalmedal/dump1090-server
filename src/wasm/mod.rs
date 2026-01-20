use super::proto::flight_service_client::FlightServiceClient;
use super::proto::{Empty, FlightData, HistoricalData, Receiver};

use std::error::Error;
use tonic_web_wasm_client::Client;

#[cfg(feature = "wasm")]
pub async fn get_flight_data(
    server_url: String,
) -> Result<tonic::Response<FlightData>, Box<dyn Error>> {
    let wasm_client = Client::new(server_url);
    let mut client = FlightServiceClient::new(wasm_client);
    let req = Empty {};
    let request = tonic::Request::new(req);
    let response = client.get_flight_data(request).await?;
    Ok(response)
}

pub async fn get_receiver_data(
    server_url: String,
) -> Result<tonic::Response<Receiver>, Box<dyn Error>> {
    let wasm_client = Client::new(server_url);
    let mut client = FlightServiceClient::new(wasm_client);
    let req = Empty {};
    let request = tonic::Request::new(req);
    let response = client.get_receiver_data(request).await?;
    Ok(response)
}

pub async fn get_historical_data(
    server_url: String,
) -> Result<tonic::Response<HistoricalData>, Box<dyn Error>> {
    let wasm_client = Client::new(server_url);
    let mut client = FlightServiceClient::new(wasm_client);
    let req = Empty {};
    let request = tonic::Request::new(req);
    let response = client.get_historical_data(request).await?;
    Ok(response)
}
