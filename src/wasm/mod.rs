use super::proto::flight_service_client::FlightServiceClient;
use super::proto::{Empty, FlightData};

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
