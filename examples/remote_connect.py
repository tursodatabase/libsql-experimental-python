"""
A short example showing how to connect to a remote libsql or Turso database

Set the LIBSQL_URL and LIBSQL_AUTH_TOKEN environment variables to point to a database.
"""
import os

import libsql

print(F"connecting to {os.getenv('LIBSQL_URL')}")
conn = libsql.connect(database=os.getenv('LIBSQL_URL'),
                      auth_token=os.getenv("LIBSQL_AUTH_TOKEN"))
conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER);")
conn.execute("INSERT INTO users(id) VALUES (10);")
conn.commit()

print(conn.execute("select * from users").fetchall())
