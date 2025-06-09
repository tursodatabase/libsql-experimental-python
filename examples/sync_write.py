"""
A short example showing how to create an embedded replica, make writes and then read them

Set the LIBSQL_URL and LIBSQL_AUTH_TOKEN environment variables to point to a database.
"""
import os

import libsql

print(F"syncing with {os.getenv('LIBSQL_URL')}")
conn = libsql.connect("hello.db", sync_url=os.getenv("LIBSQL_URL"),
                      auth_token=os.getenv("LIBSQL_AUTH_TOKEN"))
conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER);")
conn.execute("INSERT INTO users(id) VALUES (1);")
conn.commit()
conn.sync()

print(conn.execute("select * from users").fetchall())
