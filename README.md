## Lightyear investigation on rollback

Simplest example with two networked entities

- Cube (predicted, dynamic rigidbody)
- Floor (static rigidbody)


To run (server + client):

```bash
cargo run
```

To run (client 2):

```bash
cargo run client -c2 
```


For detailed logs:

```bash
RUST_LOG=debug cargo run client -c2 
```