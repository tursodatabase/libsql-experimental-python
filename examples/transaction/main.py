import libsql

conn = libsql.connect("local.db")
cur = conn.cursor()

conn.execute("DROP TABLE IF EXISTS users")
conn.execute("CREATE TABLE users (name TEXT);")
conn.execute("INSERT INTO users VALUES ('first@example.com');")
conn.execute("INSERT INTO users VALUES ('second@example.com');")

conn.rollback()

conn.execute("INSERT INTO users VALUES ('third@example.com');")

print(conn.execute("select * from users").fetchall())
