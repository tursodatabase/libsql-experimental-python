import libsql

con = libsql.connect("hello.db", sync_url="http://localhost:8080",
                                  auth_token="")

con.sync()

cur = con.cursor()

cur.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, email TEXT);")
cur.execute("INSERT INTO users VALUES (1, 'penberg@iki.fi')")

print(cur.execute("SELECT * FROM users").fetchone())
