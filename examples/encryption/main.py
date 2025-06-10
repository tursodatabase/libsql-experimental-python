import libsql

# You should set the ENCRYPTION_KEY in a environment variable
# For demo purposes, we're using a fixed key
encryption_key= "my-safe-encryption-key";

conn = libsql.connect("local.db", encryption_key=encryption_key)
cur = conn.cursor()

conn.execute("CREATE TABLE IF NOT EXISTS users (name TEXT);")
conn.execute("INSERT INTO users VALUES ('first@example.com');")
conn.execute("INSERT INTO users VALUES ('second@example.com');")
conn.execute("INSERT INTO users VALUES ('third@example.com');")


print(conn.execute("select * from users").fetchall())
