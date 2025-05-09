## Redis

#### Idea
Redis like in-memory storage implementation, for learning purposes only. Instructions, tasks, ideas were given by [CodeCrafters](codecrafters.io) (Build your own Redis course)

#### Commands
Support for most used commands such as: `GET`, `SET`, `EXPIRE`, `DEL`, `TTL`, `KEYS`, CONFIG`. Adding new command implementation is easy through Router (`src/server/router.rs`)

#### Database
All key-value pairs are stored in-memory object `Database` that is shared between all of application, and between threads. `Database` has 3 vectors (sizable arrays):
* key-value pairs (`HashMap<String, String>`)
* metadata about expiration (`HashMap<String, Metadata>`)
* all keys, used for searching (`Vec<String>`)

#### Expiration
Expiration is implemented as scheduler that runs in intervals and cleans expired objects. Exact algorithm used for selecting objects is copied from official implementation ([redis/src/expire.c](https://github.com/redis/redis/blob/a92921da135e38eedd89138e15fe9fd1ffdd9b48/src/expire.c#L98)). After each run, expiration module calculates the expired keys and sets next timer. If expired keys are less than given threshhold (for example less than 50%) module sleeps for longer than normal.

#### RDB file
At each initialization, we look for given RDB file to resurrect previous database information. RDB file is dump file written by running server periodically or by command. It includes all databases and each collection. It has its own format to encode / decode. Decoding implementation is inspired by [rdb-rs](https://github.com/badboy/rdb-rs).

#### Replication
Replication is implemented in Redis standards, meaning server can be a member to actual Redis server. At initialization server is always master, but through command, it can connect to other node via handshake marking itself as slave. After establishing connection master acknowledges its slaves and sends them its own state as RDB file.

When master receives write commands, it distributes / proxies command to each of its slaves - thus making sure to save state of data in each node. Distributer itself is connected to main Server via mpsc channels. And master has Socket connection to each of its slaves.