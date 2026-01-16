# dump1090-server

Serves tracked airplanes from dump1090-fa's aircraft.json.

## Environment variables

- GRCP_SERVER_URL - url for the gRCP server
- CERT_PATH - path for certificate
- KEY_PATH - path for private key
- ALLOW_ORIGIN - domain to add to allow origin header
- JSON_DIR - directory where aircraft.json is stored

## Run
Generate self signed certificate for localhost:

```bash
mkdir -p ./certs/local
openssl req -x509 -new -newkey rsa:4096 -nodes -keyout ./certs/local/key.pem -out ./certs/local/cert.pem -days 365 -subj "/CN=localhost" -addext "subjectAltName = DNS:localhost"
```

-  start using docker

```{bash}
docker build . -t dump1090-server
docker run -e JSON_DIR="./data" -e GRPC_SERVER_URL="0.0.0.0:50051" -e CERT_PATH="/certs/local/cert.pem" -e KEY_PATH="/certs/local/key.pem" -e ALLOW_ORIGIN="https://example.com" -v ./test-data:/data/ -v ./certs:/certs -p 50051:50051 --name dump1090-server -t dump1090-server:latest
```

Test service with grpcurl:
```
# from project folder
# list available services 
grpcurl -cacert ./certs/local/cert.pem localhost:50051 list

# Get current flight data
grpcurl -cacert ./certs/local/cert.pem -d '{}' localhost:50051 dump1090_server.FlightService/GetFlightData
```

