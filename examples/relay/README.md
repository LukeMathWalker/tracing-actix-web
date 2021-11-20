# relay across microservices

By enabling feature `relay`, you can relay `request_id` across request calls and receptions, allowing you to track a request across multiple services.

## Running

In the terminal, open 3 separate tabs, and then run the following commands:

1. Run the `microservice` first:
    ```sh
    cargo --package relay --bin microservice
    ```

2. Then run the `facade`:
    ```sh
    cargo --package relay --bin facade
    ```

3. Query the service, e.g.:
    Fetch all users:
    ```sh
    curl http://127.0.0.1:8080/users
    ```
    Fetch only male users:
    ```sh
    curl http://127.0.0.1:8080/users\?gender\=male
    ```
    Fetch only female users:
    ```sh
    curl http://127.0.0.1:8080/users\?gender\=female
    ```

Check the `request_id` in the console logs,
you'll notice that it's piped from the `facade` down to the `microservice` !

---

Please note that if you call the `microservice` directly, e.g. with [ghz](https://ghz.sh/):

```sh
ghz \
  --insecure \
  --async \
  -n 1 \
  --import-paths ./examples/relay/proto \
  --proto ./service.proto \
  --call users.Protocol.Fetch \
  127.0.0.1:50051
```

A `request_id` will still be generated for you, being consistent with `tracing-actix-web` middleware.