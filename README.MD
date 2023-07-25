
# Discord's data services from their blog post

So discord shared post(https://discord.com/blog/how-discord-stores-trillions-of-messages) how they managed to resolve problem with "hot partitions" it happened when some super popular channel on server got some discussion etc. how they resolved it is they created data-services to coalesce requests with same id and i mostly recreated them.

To test by yourself you need to:
```
docker compose up
```
to spawn a scylladb locally i suggest changing cpu and memory usage because i was to lazy to do it myself :D

create table in db:
```
CREATE TABLE messages (
   channel_id bigint,
   bucket int,
   message_id bigint,
   author_id bigint,
   content text,
   PRIMARY KEY ((channel_id, bucket), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC);
```
and then

```
cargo run --bin seed
```
and then
```
cargo build --release
```

then cd into target/release at the root of the folder and run binary

```
./request_coalescing
```

I benchmarked this myself with ghz(grpc testing tool) with requests for 5 diffrent ids and 20k requests per second(note that if you really wanna benchmark this solotuion you need to seed database and test it like in real scenario so so a lot of requests with diffrent id and one id that is requested more than another, i am to lazy to do that) so here are my results

Method without coalescing:
```
Summary:
  Count:        50000
  Total:        12.94 s
  Slowest:      345.69 ms
  Fastest:      2.35 ms
  Average:      126.12 ms
  Requests/sec: 3864.92

Response time histogram:
  2.349   [1]     |
  36.684  [2415]  |∎∎∎∎∎∎∎∎
  71.018  [6918]  |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  105.352 [10153] |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  139.686 [12294] |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  174.021 [11242] |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  208.355 [4973]  |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  242.689 [1519]  |∎∎∎∎∎
  277.023 [295]   |∎
  311.358 [101]   |
  345.692 [89]    |

```
and now method with Coalescing:
```
Summary:
  Count:        50000
  Total:        12.40 s
  Slowest:      470.51 ms
  Fastest:      1.18 ms
  Average:      87.20 ms
  Requests/sec: 4033.77

Response time histogram:
  1.177   [1]     |
  48.110  [11324] |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  95.043  [20201] |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  141.976 [12167] |∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  188.909 [4364]  |∎∎∎∎∎∎∎∎∎
  235.841 [1487]  |∎∎∎
  282.774 [366]   |∎
  329.707 [36]    |
  376.640 [20]    |
  423.573 [31]    |
  470.505 [3]     |
```
just here we can see 35 ms more response time and proly db load and with more data and requests for specifc id it will grow.

Note that in real world scenario it would be perfect to spawn multiple shard_tasks from my code according to cpu assigned in k8s and modulo hash according to channel id from schema.