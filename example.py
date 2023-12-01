import os

import libsql_experimental as libsql

print(F"syncing with {os.getenv('LIBSQL_URL')}")
conn = libsql.connect("hello_sync.db", sync_url=os.getenv("LIBSQL_URL"),
                      auth_token=os.getenv("LIBSQL_AUTH_TOKEN"))
conn.execute("CREATE TABLE IF NOT EXISTS users_sync (id INTEGER);")
conn.execute("INSERT INTO users_sync(id) VALUES (1);")
conn.commit()

print(conn.execute("select * from users_sync").fetchall())
